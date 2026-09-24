import struct
import subprocess
import sys

import msgpack


def main():
    worker, model, audio = sys.argv[1:]
    commands = [{'type': 'hello'}]
    for job in ('first', 'rerun'):
        commands.append({
            'type': 'diarize_file',
            'payload': {'job_id': job, 'path': audio, 'model_path': model},
        })
    commands.append({'type': 'shutdown'})
    frames = bytearray()
    for command in commands:
        payload = msgpack.packb({
            'protocol_version': 5,
            'request_id': 'packaged-diarization-test',
            'body': command,
        }, use_bin_type=True)
        frames.extend(struct.pack('<I', len(payload)) + payload)

    result = subprocess.run([worker], input=frames, capture_output=True, timeout=180)
    assert result.returncode == 0, f'Worker exit {result.returncode}: {result.stderr.decode()}'
    output = result.stdout
    speakers = {'first': set(), 'rerun': set()}
    completed = []
    while output:
        length = struct.unpack('<I', output[:4])[0]
        envelope = msgpack.unpackb(output[4:4 + length], raw=False)
        assert envelope['protocol_version'] == 5
        event = envelope['body']
        output = output[4 + length:]
        assert event['type'] not in ('error', 'segment'), event
        if event['type'] == 'speaker_turn':
            turn = event['payload']['turn']
            assert turn['end_ms'] > turn['start_ms']
            speakers[event['payload']['job_id']].add(turn['speaker_label'])
        if event['type'] == 'completed':
            completed.append(event['payload']['job_id'])
    assert completed == ['first', 'rerun'], completed
    assert all(speakers.values()), speakers
    print('Packaged worker passed repeated real-model diarization without loading Whisper.')


if __name__ == '__main__':
    main()
