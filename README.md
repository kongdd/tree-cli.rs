# ntree

Count the files in directories

## Features

![](./images/ntree.png)

- Count files recursively in directories
- Filter by file extension
- Skip hidden files and directories
- Display count of files in each directory

## Installation

```bash
cargo install --git https://github.com/kongdd/tree-cli.rs
```

### Usage

```bash
ntree /path/to/directory
ntree /path/to/directory --ext exe # With Extension Filter
```

```bash
ntree Catchments/Camels-SPAT/observations
Counting files in directory: Catchments/Camels-SPAT/observations
        Catchments/Camels-SPAT/observations\Forcing\headwater\rdrs-lumped | 304
        Catchments/Camels-SPAT/observations\Forcing\macro-scale\rdrs-lumped | 395
        Catchments/Camels-SPAT/observations\Forcing\meso-scale\rdrs-lumped | 727
      Catchments/Camels-SPAT/observations\headwater\obs-daily | 304
      Catchments/Camels-SPAT/observations\headwater\obs-hourly | 304
      Catchments/Camels-SPAT/observations\macro-scale\obs-hourly | 395
      Catchments/Camels-SPAT/observations\meso-scale\obs-hourly | 727
```

## References

- <https://github.com/peteretelej/tree>
