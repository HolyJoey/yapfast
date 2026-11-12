# Display name style fonts

The fonts Discord draws display name styles in, by Discord's font ids. All are
from [google/fonts](https://github.com/google/fonts) (`ofl/`), under the SIL
Open Font License 1.1; each one's licence, with its copyright line, is in
`licenses/`. They were fetched on 2026-11-11 and modified with fontTools:
variable fonts instanced at weight 600, and every font subset to Latin
(U+0020-007E, U+00A0-017F, U+2000-206F, U+20AC) to keep the binary small.

| Id | Discord's name | Font | File here |
|---|---|---|---|
| 3 | Sakura | Cherry Bomb One | `CherryBombOne-Regular.ttf` |
| 4 | Jellybean | Chicle | `Jellybean-Regular.ttf` |
| 6 | Modern | MuseoModerno | `MuseoModerno-SemiBold.ttf` |
| 8 | 8Bit | Pixelify Sans | `PixelifySans-SemiBold.ttf` |
| 12 | Tempo | Zilla Slab | `ZillaSlab-SemiBold.ttf` |
| 13 | Monkey Bars | Playpen Sans | `PlaypenSans-SemiBold.ttf` |
| 14 | Mainframe | Orbitron | `Mainframe-SemiBold.ttf` |
| 15 | Headbang | New Rocker | `Headbang-Regular.ttf` |
| 16 | Journal | Kalam | `Kalam-Bold.ttf` |

Chicle, Orbitron and New Rocker have Reserved Font Names, which a modified
version may not use, so their modified versions here are renamed after
Discord's names for the styles: Jellybean, Mainframe and Headbang.

7 Neo Castel (Medieval) and 10 Sinistre (Vampyre) are Velvetyne's and not
included: names in them show in the UI's typeface.
