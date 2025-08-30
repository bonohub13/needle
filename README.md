# Needle

Local application that displays the time! \
Uses [wgpu](https://wgpu.rs/) for rendering, [glyphon](https://github.com/grovesNL/glyphon) for text rendering, and written in Rust!

![Example of Needle running on Linux](./doc/resources/common/needle.gif)

## Installation

Go to [releases](https://github.com/bonohub13/needle/releases/latest) and download the executable.
- Windows: [needle.exe](https://github.com/bonohub13/needle/releases/download/0.2.3/needle.exe)
- Linux: [needle](https://github.com/bonohub13/needle/releases/download/0.2.3/needle)

## Instruction

- Read the [instructions](./doc/INSTRUCTIONS.md) for the following.
    1. Examples to use in OBS Studio.
    2. Background transparency settings for Windows.
- For Japanese users: [instructions_jp](./doc/INSTRUCTIONS_JP.md)

## Getting Help

This is a project that is done on my spare time. So please bear that in mind. \
If you have a feature request, please feel welcome to create an issue for feature request. \
Any requested feature that aligns with the [direction of Needle](#direction_of_needle) will be added to the [todo list](#todo_list).

## Direction of Needle <a name="direction_of_needle"></a>

Needle was originally developed for a friend of mine that does streaming on his spare time. \
He was trying to get a clock to be up on his stream, but the service he was using had the following problems.
- Crashed a LOT.
- Due to it being a web service, it was littered with ads to the point of being unusable.

To address these problems I developed Needle, so he could have a relient app that would be free from ads. \
The direction of needle consists of these criteria.
1. K.I.S.S (Keep It Simple Stupid)
2. Usability is the utmost priority.
    - Making needle a feature rich clock is great, but cluttering the app with features \
    to the point where the user feels overwelmed is not.
    - No minor features that fills a niche. (If you want it, fork it)
3. Freedom to do whatever you want.
    - Give the users the capability to do whatever they want.

## TODO List <a name="todo_list"></a>

Following features are to be added in the future. \
Features in the TODO List that have been implemented will be erased from the list after release.

- Status for individual features
    | Status                      |                                                    |
    |:----------------------------|:---------------------------------------------------|
    | In Query                    | In queue for development                           |
    | High Priority               | High priority in queue                             |
    | In Development              | Currently in development                           |
    | Next Release                | Feature implemented, and in queue for next release |
    | Implemented in VersionX.Y.Z | Feature released on Version X.Y.Z                  |

- Features in queue for development/in active development
    | Date (Added)  | Feature                                   | Status                                |
    |:--------------|:------------------------------------------|:-------------------------------------:|
    | 2024/12/22    | Transparent background for Windows        | Feature released on Version 0.2.3     |
    | 2025/02/10    | Websocket server/client                   | In Query                              |
    | 2025/02/22    | Basic PvP matchpoint counter overlay      | In Query                              |
    | 2025/02/22    | Auto-update feature (optional)            | In Query                              |
    | 2025/06/15    | Font selection                            | Feature released on Version 0.2.2     |
    | 2025/08/30    | Toggle fullscreen                         | In Query                              |
    | 2025/08/30    | Texture for background (optional)         | In Query                              |
    | 2025/08/30    | Renderers for overlays (optional)         | In Query                              |

## Naming Scheme for Branch <a name="branch_naming_scheme"></a>
- Naming convention for branches
    |Prefix             |Description                                |
    |-------------------|-------------------------------------------|
    |feature/xyz        |Branch for implementing new feature xyz    |
    |restructure/xyz    |Code restructuring module xyz              |
    |bugfix/xyz         |Fixing bugs xyz                            |
    |development/xyz    |Other development things including xyz     |
    |testing/xyz        |Testing xyz                                |
