# AXIOM CLI

The AXIOM CLI is the single entry point for the bootstrap toolchain.

## Commands

    axiom doctor
    axiom check <source.ax>
    axiom build <source.ax> [-o <output>]
    axiom run <source.ax|artifact.axm>
    axiom new <project>
    axiom test [path]
    axiom add <package> [--registry <path>]
    axiom package [-o <output>]

doctor reports the local toolchain environment.

check parses and validates a source file.

build produces the textual bootstrap IR.

run executes source or a native AXIOM artifact.

new creates a project manifest, source directory, test directory and starter program.

test validates and executes AXIOM source files below the selected test path.

add resolves a package from the local registry and records the dependency and lock state.

package creates an .axpkg archive from the current project.

The command surface is intentionally small while the compiler, runtime and package formats are still stabilizing.
