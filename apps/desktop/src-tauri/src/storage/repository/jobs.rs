use opentranscribe_domain::{Job, JobState};

use super::job_record::{self, JobRecord, JobRequest};
use super::repository::LibraryRepository;
use crate::error::{AppError, AppResult};

impl LibraryRepository {
    pub fn jobs(&self) -> AppResult<Vec<Job>> {
        Ok(job_record::list(&self.jobs_directory())?
            .into_iter()
            .map(|record| record.job)
            .filter(|job| !matches!(job.state, JobState::Completed | JobState::Canceled))
            .collect())
    }

    pub fn has_running_jobs(&self) -> AppResult<bool> {
        Ok(job_record::list(&self.jobs_directory())?
            .iter()
            .any(|record| {
                matches!(
                    record.job.state,
                    JobState::Queued | JobState::Preparing | JobState::Running
                )
            }))
    }

    pub fn has_active_job_for_session(&self, session_id: &str) -> AppResult<bool> {
        Ok(job_record::list(&self.jobs_directory())?
            .iter()
            .any(|record| {
                record.job.session_id.as_deref() == Some(session_id)
                    && matches!(
                        record.job.state,
                        JobState::Queued | JobState::Preparing | JobState::Running
                    )
            }))
    }

    pub(crate) fn create_job(
        &self,
        session_id: String,
        request: JobRequest,
    ) -> AppResult<JobRecord> {
        let duplicate = job_record::list(&self.jobs_directory())?
            .iter()
            .any(|record| {
                record.job.session_id.as_deref() == Some(&session_id)
                    && matches!(
                        record.job.state,
                        JobState::Queued | JobState::Preparing | JobState::Running
                    )
            });

        if duplicate {
            return Err(AppError::Application(
                "a transcription is already running for this session".to_owned(),
            ));
        }

        job_record::create(&self.jobs_directory(), session_id, request)
    }

    pub(crate) fn job_record(&self, job_id: &str) -> AppResult<JobRecord> {
        job_record::load(&self.jobs_directory(), job_id)
    }

    pub(crate) fn update_job(
        &self,
        job_id: &str,
        update: impl FnOnce(&mut Job),
    ) -> AppResult<JobRecord> {
        let mut record = self.job_record(job_id)?;
        update(&mut record.job);
        record.job.updated_at = opentranscribe_domain::now();
        job_record::save(&self.jobs_directory(), &record)?;
        Ok(record)
    }
}
