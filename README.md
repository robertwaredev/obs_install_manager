# OBS Install Manager

This is a simple, cross-platform TUI app which handles downloading OBS and related software for audio routing and processing. The intent behind this is to simplify the setup, configuration, and version control process of OBS for those who are not knowledgeable of the OBS ecosystem or audio software specifics. This is primarily targeted at music producers and music production teachers who need a quick and universal setup to record their music projects or lessons across across all operating systems.

## Features

### Windows

- Automatic download of the latest OBS portable installation.
- Automatic organization of newly downloaded OBS versions into separate folders which adhere to semantic versioning.
- Automatic centralization of OBS profiles and scene configurations via a symlinked folder which is shared across all OBS portable installations.

### MacOS

- Automatic download of the latest OBS installation.

### Windows & MacOS

- Automatic download of pre-configured OBS profile and scene collection for quick start. 

## How to Use

### Windows

Download and put this into a new folder specifically for OBS before running. This can be located anywhere, and named anything, but my recommendation is creating a folder named "OBS" in your "Documents" folder so that you can find it easily.

### MacOS

Just run it with Terminal.

## Planned Features

- OBS version selection as opposed to only the latest, greatest version.
- Potentially offering multiple OBS configurations suited to different use cases.
