# rmosaic

Generates [photographic mosaics](https://en.wikipedia.org/wiki/Photographic_mosaic).

## Usage

```
USAGE:
    rmosaic [FLAGS] [OPTIONS] <INPUT> <LIBRARY> <OUT_FILE>

FLAGS:
    -c, --color-caching    Enables caching of closest-color matches. May improve performance if the input image has many
                           identical colors repeated and/or you have a large image library.
    -h, --help             Prints help information
    -t, --print-timings    Print timings
    -V, --version          Prints version information
    -v, --verbose          Sets the level of verbosity

OPTIONS:
    -m, --magnification <MAGNIFICATION_FACTOR>
            The integer factor by which the original image's dimensions are increased. Defaults to 2

    -g, --pixel-group-size <SIZE>
            The integer size of the square regions, in pixels, which will be replaced in the source image. Defaults to
            16
        --threads <THREADS>                       The number of threads used. Defaults to 2

ARGS:
    <INPUT>       Sets the input file which will be recreated as a mosaic
    <LIBRARY>     The directory containing the images to be used as mosaic tiles
    <OUT_FILE>    The name of the output mosaic image
```

## Sample

```
rmosaic sample/last_supper/in.jpg sample/solid_colors sample/last_supper/out.png -g 1 -m 1
```

![Last Supper input image](https://raw.githubusercontent.com/ipost/mosaic-rs/master/sample/last_supper/in.jpg)
![Last Supper output image](https://raw.githubusercontent.com/ipost/mosaic-rs/master/sample/last_supper/out.png)
