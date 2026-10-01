# AXIOM CLI

The native axiom binary is the reference entry point for the independent
toolchain. The historical Python CLI remains available for compatibility and
development tooling.

## Commands

    axiom doctor
    axiom new <project>
    axiom check <source.ax|project>
    axiom build <source.ax|project> [-o <output>]
    axiom run <source.ax|project|artifact.axm>
    axiom test <project>
    axiom package <project> [-o <output>]
    axiom add <project> <package> [registry]
    axiom native-build <source.ax|project> [-o <output>]
    axiom install <prefix>

doctor reports the native toolchain environment.

new creates a project with a manifest, source and test.

check validates an AXIOM source file or project entry point.

build produces AXIOM_ARTIFACT_V1.

run executes source, project or AXIOM artifact.

test executes every .ax test below the project's tests directory.

package creates deterministic AXIOM_PACKAGE_V1 output.

add vendors a package from a local registry and writes axiom.lock.

native-build creates a standalone executable using the selected native target.

install copies the running native axiom binary into a prefix/bin directory.

The native workflow does not require Python.
