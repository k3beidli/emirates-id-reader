# README animation

`card-reader.gif` is a conversion of the project's existing card-reader animation,
`emirates_id_insert_alpha.webm`, rendered from `animate_card_insert.py`.
The source artwork uses a sample card with placeholder identity values.

The GIF loops at 24 fps, 320 × 400, with transparency. To regenerate from the
original 800 × 1200 alpha WebM, run FFmpeg from the repository root:

```sh
ffmpeg -y -c:v libvpx-vp9 -i emirates_id_insert_alpha.webm -filter_complex "[0:v]fps=24,crop=800:1000:0:200,scale=320:400:flags=lanczos,split[a][b];[a]palettegen=max_colors=128:reserve_transparent=1:stats_mode=diff[p];[b][p]paletteuse=dither=bayer:bayer_scale=3:alpha_threshold=128" -loop 0 docs/media/card-reader.gif
```

The original animation project/video is maintained separately and is not required
to build the SDK or sample application.
