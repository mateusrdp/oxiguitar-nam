# NAM plugin

This plugin plays a guitar or bass track through an amp model. A model is a capture of a real amplifier, pedal or whole rig made with [Neural Amp Modeler](https://github.com/sdatkinson/neural-amp-modeler) and stored in a `.nam` file. After the amp, the sound goes through a cabinet, which is an impulse response of a speaker and its microphone.

You set it per track in the track dialog's **Amp (NAM)** section. Models and cabinets are files you get yourself and put in a folder. The plugin downloads nothing.

This work depends on the MIDI plugin, which makes the sound the amp shapes. See [its manual](https://mateusrdp.github.io/oxiguitar-midi/).
