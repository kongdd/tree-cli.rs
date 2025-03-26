# File Counter

A utility for counting files in directories.

避免了一个个数文件的困扰。

## Overview

File Counter is a command-line tool designed to help you analyze your file system by counting files. This project is currently under development.

## Features

- Count files recursively in directories
- Filter by file extension
- Skip hidden files and directories
- Display count of files in each directory

## Usage

```bash
cargo install --git https://github.com/kongdd/file-counter
```

### Basic Usage

```bash
file-counter /path/to/directory
```

### With Extension Filter

```bash
file-counter /path/to/directory --ext exe
```

```bash
file-counter.exe Catchments/Camels-SPAT/observations
Counting files in directory: Catchments/Camels-SPAT/observations
        Catchments/Camels-SPAT/observations\Forcing\headwater\rdrs-lumped | 304
        Catchments/Camels-SPAT/observations\Forcing\macro-scale\rdrs-lumped | 395
        Catchments/Camels-SPAT/observations\Forcing\meso-scale\rdrs-lumped | 727
      Catchments/Camels-SPAT/observations\headwater\obs-daily | 304
      Catchments/Camels-SPAT/observations\headwater\obs-hourly | 304
      Catchments/Camels-SPAT/observations\macro-scale\obs-hourly | 395
      Catchments/Camels-SPAT/observations\meso-scale\obs-hourly | 727
```
