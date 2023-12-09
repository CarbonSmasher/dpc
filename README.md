# DPC
DPC is essentially just LLVM for Minecraft datapacks: a compilation backend that converts a common input language to Minecraft function files.

## Architecture
DPC begins by processing IR, which is either given in code when used as a library, or parsed from text. The text representation looks (will look) something like this:

```
"foo:main/main" {
	let x: score = 7;
	let y: score = x;
	sub y, x;
	if eq x, 8: call "foo:main/bar";
}

"foo:main/bar" {
	say "hello";
}
```
The IR will have support for:
 - Registers with different data types
 - Function calls with return values and arguments
 - If/else logic
 - Execute modifiers
 - Loops

IR is designed to be as simple as possible to create and have a stable interface between updates, but it is not always the easiest to optimize. Thus, IR is quickly lowered to MIR, a very similar format which has slightly less convenience, but is where most of the optimizations happen. Inside MIR, multiple passes run over the instructions such as:

 - Function inlining
 - Loop optimizations
 - Math optimizations
 - Combination of modifiers
 - Logic optimizations
 - Macro optimizations

Next, MIR is lowered one more time to LIR, which is as close as possible to actual commands. While some IR and MIR instructions might end up representing more than one command, LIR instructions are pretty much 1:1. Another difference is the representation of types. While IR and MIR instructions are generic across multiple types and just give errors for types they don't support, LIR has separate instructions for different types of data. In LIR, more passes are run that are more Minecraft-specific, such as:

 - Reordering selector parameters
 - Optimizing execute modifiers

Finally, the LIR is processed into the most optimal versions of Minecraft commands.

## Progress
Right now, the main thing that needs to be done is the implementation of all the commands in the game as instructions. That, along with fleshing out other optimizations and features like custom instructions, macros, and overlays.
