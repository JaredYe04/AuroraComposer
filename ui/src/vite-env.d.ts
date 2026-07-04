/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue';
  const component: DefineComponent<object, object, unknown>;
  export default component;
}

declare module 'abcjs';
declare module 'verovio/wasm';
declare module 'verovio/esm';

declare module 'soundfont-player' {
  interface PlayOptions {
    duration?: number;
    gain?: number;
    attack?: number;
    decay?: number;
    sustain?: number;
    release?: number;
  }

  interface InstrumentPlayer {
    play(
      note: number | string,
      when?: number,
      options?: PlayOptions,
    ): { stop: (when?: number) => void };
    stop(when?: number, nodes?: unknown): unknown[];
  }

  interface InstrumentOptions {
    soundfont?: string;
    format?: string;
    destination?: AudioNode;
    notes?: number[] | string[];
    nameToUrl?: (name: string, soundfont?: string, format?: string) => string;
  }

  function instrument(
    ac: AudioContext,
    name: string,
    options?: InstrumentOptions,
  ): Promise<InstrumentPlayer>;

  export default { instrument };
}
