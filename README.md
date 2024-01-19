# DPC
DPC is essentially just LLVM for Minecraft datapacks: a compilation backend that converts a common input language to Minecraft function files. It is not meant to be used as a high level language that you write datapacks in directly. Instead, a frontend language with more useful features will create IR that this project can easily optimize and turn into a pack for the targeted version.

## Architecture
DPC begins by processing IR, which is either given in code when used as a library, or parsed from text. The text representation looks (will look) something like this:

```
"foo:main/main" {
	let x: score = val 7s;
	let y: score = %x;
	sub %y, %x;
	if eq %x, 8s: call run "foo:main/bar";
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
 - Removing dead code
 - Merging/caching NBT operations

Next, MIR is lowered one more time to LIR, which is as close as possible to actual commands. While some IR and MIR instructions might end up representing more than one command, LIR instructions are pretty much 1:1. Another difference is the representation of types. While IR and MIR instructions are generic across multiple types and just give errors for types they don't support, LIR has separate instructions for different types of data. In LIR, more passes are run that are more Minecraft-specific, such as:

 - Reordering selector parameters
 - Optimizing execute modifiers
 - Reformatting for execute store
 - Preparing values for codegen, depending on the target

Finally, the LIR is processed into Minecraft commands in the codegen stage. Codegen only looks at one instruction at a time, selecting the best and smallest variants to reduce output size.

## Progress
Right now, the main thing that needs to be done is the implementation of all the commands in the game as instructions. That, along with fleshing out other optimizations and features like custom instructions, macros, and overlays.

 - [ ] IR features
   - [x] Types
   - [x] NBT types and instructions
   - [x] Registers
   - [x] Functions with annotations
   - [x] Arguments and return values
   - [x] Math and logical instructions
   - [x] Custom commands
     - [ ] Custom command access to registers, arguments, etc.
   - [x] If
   - [x] Nested blocks
   - [ ] Loops
   - [ ] Lookup tables
   - [x] Else
   - [ ] Inverse binops
   - [ ] Function tag declaration
   - [ ] Recursion
   - [ ] Macro types
   - [ ] Global storage
   - [ ] Structural types
   - [ ] Return propagation
 - [ ] Minecraft instructions and features
   - [ ] Command instructions
     - [x] Most commands
     - [ ] `/tellraw` and `/title`
     - [ ] `/damage`
     - [ ] `/random`
     - [ ] `/advancement` and `/recipe`
     - [ ] `/schedule`
     - [ ] `/particle`
     - [ ] `/bossbar`
     - [ ] `/team modify`, `/scoreboard objectives/players modify`
   - [ ] Some selector parameters
   - [ ] Some modifiers
 - [ ] Compilation / usage
   - [x] Datapack target
   - [x] Fine control over passes run
   - [ ] Command block target
   - [ ] Version targeting
   - [ ] Overlays
   - [ ] Automatically inserted overflow checks / assertions
   - [ ] Debugging / breakpoint instructions
 - [ ] Optimizations
   - [x] Instruction simplification
   - [x] Function inlining
   - [x] Dead store elimination
   - [x] Dead code elimination
   - [x] Constant propagation, folding, and evaluation
   - [x] Function simplification
   - [x] Smart register allocation
   - [x] Scoreboard dataflow
   - [ ] Instruction pattern matching
     - [x] Instruction combining
     - [x] Assignment pattern matching
     - [ ] Math pattern matching
     - [ ] Conditional pattern matching
     - [x] Logical pattern matching
   - [ ] Function specialization
   - [ ] Smarter / less aggressive inliner
   - [ ] Macro optimizations
   - [x] Modifier merging
   - [x] Modifier simplification
   - [x] Cost analysis
   - [ ] Modifier combination
   - [x] Null modifier removal
   - [ ] Modifier-selector optimizations
   - [ ] NBT access merging / caching
   - [ ] Value numbering
   - [ ] Type narrowing
   - [ ] Range propagation
   - [ ] Tail call optimization
   - [ ] Function merging
   - [ ] Loop optimizations
   - [ ] Advanced selector optimizations
   - [ ] Block operation combination
   - [ ] Control flow optimizations
   - [ ] Argument / return value copy elision
 - [ ] Utilities
   - [x] Parsing for most of the IR features (mostly used for testing)
   - [x] Codegen testing suite
   - [x] Basic Python bindings
   - [ ] Datapack frontend (from something like mecha) to allow datapack -> datapack compilation
