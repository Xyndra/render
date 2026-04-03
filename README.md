# Project Render (no release name yet)

The goal of this project is to fix the current UI rendering scene. Bundeling a browser engine or similar is overkill for pretty much everything. 
But why do people still do it? Two reasons:

1. Keeping most of the codebase the same
    - Everyone needs a website, so why not use the website as the UI?
    - Only JavaScript really feels good when used in the web, but JS can't be compiled to native
2. Native is a headache 
    - Every platform has its quirks
    - Having a nice-looking UI is very difficult

Here is the plan:

1. Just have native get good performance on web
2. Build out a way to create good-looking UIs on native

_Of course, there is also the issue of language difficulty, there is the issue of Rust, but that's just a skill issue_

Goals of the project:
- Dynamic, nice to use, Component-based UI system
- Use HTML and CSS(!) for the web, but using some quirks to have it work the same way as native
    - Manual layouting using absolute positioning or possibly CSS depending on what is wanted
        - Whilst manual layouting is easier, it has to be calculated per-resize and per-change, which might become a problem 
    - Shaders using a custom system for the web to directly interface with webgpu or webgl
    - Possibility of generating manual JavaScript for the web
- For native, generate shapes and then use a harfbuzz binding and a high-performance SVG rendering engine

If this project succeeds, I will also create a standard library for things like save storage, networking, and other things that JS has an easy version for to simplify the development further.
