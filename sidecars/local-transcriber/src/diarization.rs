use std::path::Path;

use parakeet_rs::sortformer::{Sortformer, SpeakerSegment};

use crate::runtime::Segment;

pub fn diarize(path: &Path, samples: &[f32]) -> Result<Vec<SpeakerSegment>, String> {
    let mut diarizer = Sortformer::new(path).map_err(|error| error.to_string())?;
    let mut turns = diarizer
        .diarize(samples.to_vec(), 16_000, 1)
        .map_err(|error| error.to_string())?;
    turns.sort_by_key(|turn| (turn.start, turn.speaker_id));
    Ok(turns)
}

pub fn assign_speakers(segments: &mut [Segment], turns: &[SpeakerSegment]) {
    let mut next_turn = 0;
    let mut active: Vec<&SpeakerSegment> = Vec::new();

    for segment in segments {
        // Nemotron offsets are 16 kHz samples; Whisper timestamps are milliseconds.
        let start = segment.start_ms * 16;
        let end = (segment.end_ms * 16).max(start + 1);
        active.retain(|turn| turn.end > start);
        while next_turn < turns.len() && turns[next_turn].start < end {
            if turns[next_turn].end > start {
                active.push(&turns[next_turn]);
            }
            next_turn += 1;
        }

        let mut overlap = [0_u64; 8];
        for turn in &active {
            overlap[turn.speaker_id] += end.min(turn.end).saturating_sub(start.max(turn.start));
        }
        let mut best_overlap = 0;
        segment.speaker_label = None;
        for (speaker, duration) in overlap.into_iter().enumerate() {
            // Equal overlap keeps the earlier-arriving speaker. Unmatched words stay unassigned.
            if duration > best_overlap {
                best_overlap = duration;
                segment.speaker_label = Some((speaker + 1).to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SpeakerSegment, assign_speakers};
    use crate::runtime::Segment;

    fn word(start_ms: u64, end_ms: u64) -> Segment {
        Segment {
            start_ms,
            end_ms,
            text: "word".to_owned(),
            speaker_label: None,
        }
    }

    fn turn(start_ms: u64, end_ms: u64, speaker_id: usize) -> SpeakerSegment {
        SpeakerSegment {
            start: start_ms * 16,
            end: end_ms * 16,
            speaker_id,
        }
    }

    #[test]
    fn aligns_words_across_speaker_changes_and_silence() {
        let mut words = [
            word(100, 300),
            word(400, 700),
            word(800, 900),
            word(1_000, 1_100),
        ];
        assign_speakers(
            &mut words,
            &[turn(0, 450, 0), turn(450, 750, 1), turn(1_000, 1_200, 0)],
        );
        assert_eq!(
            words
                .iter()
                .map(|word| word.speaker_label.as_deref())
                .collect::<Vec<_>>(),
            vec![Some("1"), Some("2"), None, Some("1")]
        );
        assert_eq!((words[1].start_ms, words[1].end_ms), (400, 700));
    }

    #[test]
    fn uses_total_overlap_and_stable_ties_for_simultaneous_speech() {
        let mut words = [word(0, 100), word(200, 300)];
        assign_speakers(
            &mut words,
            &[
                turn(0, 30, 0),
                turn(0, 50, 1),
                turn(40, 100, 0),
                turn(200, 300, 0),
                turn(200, 300, 1),
            ],
        );
        assert_eq!(words[0].speaker_label.as_deref(), Some("1"));
        assert_eq!(words[1].speaker_label.as_deref(), Some("1"));
    }

    #[test]
    fn handles_point_timestamps_eight_speakers_and_empty_diarization() {
        let mut words = [word(100, 100), word(200, 200)];
        assign_speakers(&mut words, &[turn(0, 200, 7)]);
        assert_eq!(words[0].speaker_label.as_deref(), Some("8"));
        assert_eq!(words[1].speaker_label, None);
        assign_speakers(&mut words, &[]);
        assert!(words.iter().all(|word| word.speaker_label.is_none()));
    }
}
