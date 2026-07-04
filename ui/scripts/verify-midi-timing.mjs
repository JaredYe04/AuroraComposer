import pkg from '@tonejs/midi';
import { execFileSync } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const { Midi } = pkg;
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const outPath = path.join(__dirname, '../../target/test-export.mid');

execFileSync(
  'cargo',
  ['test', 'write_midi_fixture', '--quiet', '--', '--ignored', '--nocapture'],
  { cwd: path.join(__dirname, '../..'), stdio: 'inherit' },
);

if (!fs.existsSync(outPath)) {
  console.error('Missing fixture:', outPath);
  process.exit(1);
}

const bytes = fs.readFileSync(outPath);
const midi = new Midi(bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength));

function noteTimes(channel, midiNum) {
  const out = [];
  for (const track of midi.tracks) {
    if (track.channel !== channel && track.channel !== channel + 1) continue;
    for (const note of track.notes) {
      if (note.midi === midiNum) out.push(Number(note.time.toFixed(4)));
    }
  }
  return out.sort((a, b) => a - b);
}

const bpm = midi.header.tempos[0]?.bpm ?? 120;
const kicks = noteTimes(9, 36);
const hats = noteTimes(9, 42);
const melody = midi.tracks.find((t) => t.name === 'Melody')?.notes ?? [];
const melodyOnsets = melody.map((n) => Number(n.time.toFixed(4))).sort((a, b) => a - b);

console.log('BPM:', bpm);
console.log('Kick times (s):', kicks.slice(0, 8));
console.log('Hi-hat times (s):', hats.slice(0, 8));
console.log('Melody onsets (s):', melodyOnsets.slice(0, 8));

const beatSec = 60 / bpm;
const kick0 = kicks[0] ?? -1;
const kick1 = kicks[1] ?? -1;
const expectedKick1 = beatSec * 2; // beat 3 in 4/4 = 2 quarter notes

let ok = true;
if (Math.abs(kick0) > 0.001) {
  console.error('FAIL: first kick not at t=0, got', kick0);
  ok = false;
}
if (Math.abs(kick1 - expectedKick1) > 0.02) {
  console.error(`FAIL: second kick expected ~${expectedKick1.toFixed(3)}s, got ${kick1}`);
  ok = false;
}
if (Math.abs((hats[1] ?? 0) - beatSec / 4) > 0.02) {
  console.error('FAIL: second hi-hat not on 16th grid');
  ok = false;
}
if (Math.abs((melodyOnsets[0] ?? -1)) > 0.02) {
  console.error('FAIL: melody not starting at downbeat');
  ok = false;
}

console.log(ok ? 'PASS: @tonejs/midi timing aligned' : 'FAIL');
process.exit(ok ? 0 : 1);
