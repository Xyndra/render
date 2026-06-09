# Crate structure

```mermaid
flowchart TD
    main["render (main)"] <--> platform_opts["render platform options"]
    main <--> components["render components"]
    components <--> events["render events"]
    main <--> Platforms
    subgraph Platforms
        linux["render linux"] <--> winit
        linux <--> wgpu
        wgpu <--> winit
    end
    Platforms <--> events
    Platforms <--> layout
    Platforms <--> platform_opts
    Platforms <--> components
    components <--> layout
    layout <--> events
    layout["render layout"]
```

# Algorithm

Rules for elements:
- It must work no matter which size it is given. The parent is responsible for giving it the right size. If it is too small or too big, it may display a red box to indicate that or error but **never panic**!
- There is no such thing as absolute positioning. If you want something to appear outside the current element, go to the element it should appear in! Send some kind of signal there, like a global boolean flag. If this project grows big enough, a signaling system will be implemented, but for now, just go and do it manually.

```mermaid
flowchart TD
    subgraph Events
        Redraw
        Click["Positioned events:\n Click, Hover, Touch, Stylus, ..."]
        Keyboard["Positionless events:\n Keyboard, Controller, ..."]
    end
    Computed{Layout Already Computed?} -- Yes --> Load["Load stored"]
    Computed -- No --> Layout
    Redraw --> Computed
    subgraph Layout
        direction TB
        Kind["Determine layout kind"] -- Absolute Layout --> Position
        Position["Use given positions"] --> Size
        Size["If wanted, grow (up to the edges)"] --> Save
        Save["Save Layout to Element"]
    end
    Load <--> Save
    Load --> Collect
    Save --> Collect
    Collect["Collect Primitives"] --> Pass["Pass to Platform"]

    Click --> Find
    Find["Find Subelements "] <--> Save
    Find <-- recursive --> Passthrough
    Passthrough --> Consume["Consume Click/..."]

    Keyboard --> Global["Global Keyboard/... State"]
    App -- register callbacks --> Global
    Global --> Execute["Execute registered callbacks"]
```
