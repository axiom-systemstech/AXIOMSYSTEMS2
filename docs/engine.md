# AXIOM Engine

AXIOM Engine is the general-purpose execution foundation established during the historical language roadmap.

## Current role

The current Engine model provides backend-neutral concepts for:

- applications;
- entities and components;
- resources;
- events;
- scenes;
- simulation clocks;
- networking;
- rendering queues;
- UI queues;
- audio queues.

Its purpose is to avoid binding the language to a single graphics, physics, audio, UI, or networking implementation.

## Relation to AXIOM 0.1

The definitive AXIOM semantic model goes deeper than an application engine.

Engine concepts must eventually map to the same language-wide entities, resources, capabilities, effects, restrictions, contracts, and execution plans used by systems programming, embedded work, scientific computing, robotics, distributed systems, and hardware control.

Engine is therefore a subsystem, not the definition of the language.

## Future direction

Concrete graphics, physics, audio, UI, robotics, simulation, and hardware systems can consume AXIOM execution plans without creating separate programming languages.
