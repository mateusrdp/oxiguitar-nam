# What you can do with it

- Make a clean guitar sound like a real amp. Pick a model for a track and press Play.
- Choose a cabinet. Four come built in, or use any `.wav` impulse response from your `nam` folder. A model that includes its own cabinet needs none.
- Drive the amp harder or softer with **Input**, and set the track's level after it with **Output**, from −24 to +24 dB. Both work while the song plays, so you can set the drive by ear.
- Use different amps on different tracks.

## What to feed it

An amp model expects a clean guitar signal. Any guitar sound plays through it, but a clean sound works best. A track whose instrument is already distorted should have its **Sound** set to follow the clean guitars row, so the amp isn't distorting a distorted sound.

## Where models go

Put `.nam` and `.wav` files in the `nam` folder, up to five folders deep:

| System | The `nam` folder |
| --- | --- |
| Linux | `~/.local/share/oxiguitar/nam/` |
| macOS | `~/Library/Application Support/OxiGuitar/nam/` |
| Windows | `%APPDATA%\OxiGuitar\nam\` |
| OxiGuitar's Flatpak | `~/.var/app/io.github.mateusrdp.OxiGuitar/data/oxiguitar/nam/` |

Thousands of free models are shared at [TONE3000](https://www.tone3000.com/). Each one states its licence.

If a track's amp is on and a folder is empty, a dialog says so once. The built-in cabinets still work without any impulse response of your own.
