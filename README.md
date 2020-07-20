# game-narrative-script

A toolkit to declaratively specific non-linear narration for games and easily integrate with Godot.

### Example
1. Clone the repository
```
git clone git@github.com:liquidscorpio/game-narrative-script.git
```

2. Build using the standard cargo utilities
```
cargo build [--release]
```

3. Define your dialogues in a file
```
# In the file dialogue.gcs
:character Radius{name: "Radius"}
:character DrBread{name: "Dr. Bread"}

:act c1_intro

%define c1_intro {
    @Radius "My brother Tangent is hurt. Dr. Bread, please help."
    @DrBread "Hmm... The wound is grave and deep. I fear there is not much we can do here."
    @Radius [
        "Please, Doctor! There has to be something we can do." c1_please_help,
        "**Weep**" c1_weep,
    ]
}

%define c1_please_help {
    @DrBread "Seek out the old granny by the oak forest."
}

%define c1_weep {
    @DrBread "Do not loose hope, kid. Let me think."
}

```

4. Compile the dialogue file with the generated binary
```
cargo run dialogue.gcs
```
This will generate will generate the compiled tree and index files.
```
dialogue.gcsindex
dialogue.gcstree
```

### Integrating with Godot
We can use [gdnative](https://crates.io/crates/gdnative) to generate bindings for and directly access the tree walking API from GD script.
