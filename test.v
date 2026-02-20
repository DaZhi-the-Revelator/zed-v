#!/usr/bin/env -S v run

// test.v — Manual feature test for V Enhanced (v-enhanced Zed extension)
//
// HOW TO USE:
//   Open this file in Zed with the v-enhanced dev extension active.
//   Work through each section and verify the listed behaviour.
//   A ✅ comment marks what you should see / be able to do at that line.
//
// REQUIREMENTS:
//   v-analyzer must be in PATH and fully indexed before LSP features appear.

module main

import os

import math
import strings as str

// ============================================================
// CONSTANTS
// ✅ Outline panel shows: max_items, version, tolerance (under Constants)
// ✅ Hover on each name shows type + value
// ✅ Inlay hint: type shown after each constant (e.g. "max_items: int")
// ============================================================

const max_items = 100
const version = '1.0.0'
const tolerance = 0.001

// ============================================================
// GLOBALS  (__global)
// ✅ Hover shows type; highlights all read/write sites
// ============================================================

__global (
	request_count int
	last_error    string
)

// ============================================================
// TYPE ALIASES & SUM TYPES
// ✅ Outline panel shows: Meters, Kilograms, Payload (under Types)
// ✅ Go-to-definition works from usage sites below
// ✅ Hover on Payload shows the full sum type signature
// ============================================================

type Meters = f64
type Kilograms = f64
type Payload = string | int | f64

// ============================================================
// ENUMS
// ✅ Outline panel shows: Direction, Status with all fields nested
// ✅ Hover on Direction shows doc comment + field list
// ✅ Inlay hints: implicit field values shown (north=0, south=1, …)
// ✅ [flag] attribute enables bitfield enum — code action "Add [flag]" available on plain enum
// ============================================================

// A cardinal direction.
enum Direction {
	north
	south
	east
	west
}

@[flag]
enum Permission {
	read
	write
	execute
}

enum Status {
	ok        = 200
	not_found = 404
	error     = 500
}

// ============================================================
// INTERFACES
// ✅ Outline panel shows: Shape, Drawable with methods nested
// ✅ Code lens: "N implementations" button above each interface
// ✅ Go-to-implementation: from Shape → finds Rectangle, Circle
// ✅ Hover on area() shows signature + doc comment
// ============================================================

// Shape is anything with area and perimeter.
interface Shape {
	// Returns the area of the shape.
	area() f64
	perimeter() f64
}

interface Drawable {
	draw()
}

// ============================================================
// STRUCTS
// ✅ Outline panel shows: Point, Rectangle, Circle with fields nested
// ✅ Code lens: "implement N interfaces" button above each struct
// ✅ Go-to-implementation: from Shape interface → finds Rectangle, Circle
// ✅ Hover on Rectangle shows fields with types and mutability
// ✅ Diagnostics: rename a field to an unused name → "unused field" warning
// ✅ Code action: "Make Public" available on any private declaration
// ✅ Code action: "Add [heap]" available on struct declaration
// ============================================================

struct Point {
pub mut:
	x f64
	y f64
}

struct Rectangle {
pub:
	origin Point
pub mut:
	width  f64
	height f64
}

struct Circle {
pub mut:
	center Point
	radius f64
}

// ============================================================
// METHODS
// ✅ Outline panel shows methods nested under their receiver type
// ✅ Go-to-definition from call site (e.g. rect.area()) → jumps here
// ✅ Hover shows full signature + receiver type
// ✅ Code lens: "implement N interfaces" button above Rectangle methods
// ✅ Inlay hints: parameter names shown at call sites
// ============================================================

fn (r Rectangle) area() f64 {
	return r.width * r.height
}

fn (r Rectangle) perimeter() f64 {
	return 2.0 * (r.width + r.height)
}

fn (mut r Rectangle) scale(factor f64) {
	r.width *= factor
	r.height *= factor
}

fn (r Rectangle) draw() {
	println('Rectangle(${r.width}x${r.height})')
}

fn (c Circle) area() f64 {
	return math.pi * c.radius * c.radius
}

fn (c Circle) perimeter() f64 {
	return 2.0 * math.pi * c.radius
}

fn (c Circle) draw() {
	println('Circle(r=${c.radius})')
}

fn (p Point) distance_to(other Point) f64 {
	dx := p.x - other.x
	dy := p.y - other.y
	return math.sqrt(dx * dx + dy * dy)
}

// ============================================================
// FUNCTIONS — plain, result, option, generic, variadic
// ✅ Outline + tags panel lists all functions (Ctrl/Cmd+T to search)
// ✅ Signature help fires as you type the opening ( of each call
// ✅ Inlay hints: parameter name hints at every call site
// ✅ Hover shows full signature, doc comment, and module name
// ============================================================

// greet returns a personalised greeting.
pub fn greet(name string) string {
	return 'Hello, ${name}!'
}

// safe_divide returns an error if b is zero.
fn safe_divide(a f64, b f64) !f64 {
	if b == 0.0 {
		return error('division by zero')
	}
	return a / b
}

// find_item returns none when the item is missing.
fn find_item(haystack []string, needle string) ?string {
	for s in haystack {
		if s == needle {
			return s
		}
	}
	return none
}

// largest works on any ordered type.
fn largest[T](a T, b T) T {
	if a > b {
		return a
	}
	return b
}

// total accepts any number of ints.
fn total(nums ...int) int {
	mut acc := 0
	for n in nums {
		acc += n
	}
	return acc
}

fn multi_return() (int, string, bool) {
	return 1, 'one', true
}

// ============================================================
// ATTRIBUTES
// ✅ Hover on fast_square: shows [inline] attribute
// ✅ Hover on old_api: shows deprecation message with strikethrough in editor
// ✅ Diagnostics: calling old_api anywhere shows a deprecated warning
// ============================================================

@[inline]
fn fast_square(n int) int {
	return n * n
}

@[deprecated: 'use fast_square instead']
fn old_square(n int) int {
	return n * n
}

// ============================================================
// DIAGNOSTICS TRIGGERS
// ✅ Uncomment each block one at a time to observe the diagnostic.
//
// UNUSED VARIABLE (strikethrough + warning):
//   fn unused_var_demo() {
//       unused := 42      // ← "unused variable: unused"
//   }
//
// UNUSED IMPORT — add "import json" at the top and never use it.
//
// IMMUTABLE MUTATION (error + code action "Make Mutable"):
   //fn immutable_demo() {
   //    x := 5
   //    x = 10            // ← error; code action appears
   //}
//
// DEPRECATED (strikethrough warning):
//   fn deprecated_demo() {
//       _ := old_square(3) // ← deprecated warning
//   }
//
// UNDEFINED IDENT (error + code action "Import Module"):
//   fn missing_import_demo() {
//       data := json.encode('hi') // ← "undefined ident: json"
//   }
// ============================================================

// ============================================================
// INLAY HINTS
// ✅ Type hints — hover or look for ": type" after each :=
// ✅ Parameter name hints — look for "name:" before each argument below
// ✅ Range hints — look for ≤ / < annotations on .. ranges
// ✅ Implicit err hint — look for "err →" inside or {} blocks
// ✅ Enum field value hints — look for "= 0", "= 1" next to Direction fields above
// ✅ Constant type hints — look for ": int", ": string" next to const declarations above
// ============================================================

fn inlay_hints_demo() {
	// Type hint: x shown as "x: int"
	x := 42

	// Type hint: msg shown as "msg: string"
	msg := greet('world')

	// Parameter name hints: "name:" shown before 'Alice'
	_ := greet('Alice')

	// Range hint: ≤ shown on the .. range
	for i in 0 .. 5 {
		_ = i
	}

	// Implicit err hint: "err →" shown inside the or block
	result := safe_divide(10.0, 2.0) or {
		eprintln(err)
		0.0
	}

	_ = x
	_ = msg
	_ = result
}

// ============================================================
// CODE LENS
// ✅ "▶ Run workspace" and "▶ single file" appear above fn main()
// ✅ "▶ Run test" appears above each fn test_*
// ✅ "all file tests" appears above the first test function
// ✅ "N implementations" appears above Shape / Drawable interfaces above
// ✅ "implement N interfaces" appears above Rectangle / Circle structs above
// ============================================================

fn test_greet() {
	assert greet('Zed') == 'Hello, Zed!'
}

fn test_safe_divide() {
	result := safe_divide(10.0, 2.0) or { 0.0 }
	assert result == 5.0
}

fn test_safe_divide_by_zero() {
	result := safe_divide(1.0, 0.0) or { -1.0 }
	assert result == -1.0
}

fn test_find_item() {
	found := find_item(['a', 'b', 'c'], 'b') or { '' }
	assert found == 'b'
	missing := find_item(['a', 'b'], 'z') or { 'default' }
	assert missing == 'default'
}

fn test_largest() {
	assert largest(3, 7) == 7
	assert largest('apple', 'banana') == 'banana'
}

fn test_total() {
	assert total(1, 2, 3, 4, 5) == 15
	assert total() == 0
}

fn test_rectangle_area() {
	r := Rectangle{
		width:  4.0
		height: 3.0
	}
	assert r.area() == 12.0
}

fn test_circle_area() {
	c := Circle{
		radius: 1.0
	}
	assert math.abs(c.area() - math.pi) < tolerance
}

// ============================================================
// FOLDING
// ✅ Each { } block below should be independently foldable:
//   fn body, struct body, if block, else block,
//   for body, match body, interface body, enum body
// ============================================================

fn folding_demo() {
	x := 10

	if x > 5 {
		println('big')
	} else {
		println('small')
	}

	for i in 0 .. 3 {
		println(i)
	}

	match x {
		1 { println('one') }
		2 { println('two') }
		else { println('other') }
	}

	defer {
		println('cleanup')
	}
}

// ============================================================
// DOCUMENT SYMBOLS / OUTLINE
// ✅ Open the outline panel (View → Outline) and verify you see:
//   - Constants: max_items, version, tolerance
//   - Types: Meters, Kilograms, Payload
//   - Enums: Direction (north/south/east/west), Permission, Status
//   - Interfaces: Shape (area, perimeter), Drawable (draw)
//   - Structs: Point (x,y), Rectangle (origin,width,height), Circle (center,radius)
//   - Functions: greet, safe_divide, find_item, largest, total, multi_return, ...
//   - Methods nested under their receivers
// ============================================================

// ============================================================
// WORKSPACE SYMBOLS (Ctrl/Cmd+T)
// ✅ Searching "Rectangle" finds the struct declaration
// ✅ Searching "area" finds both Rectangle.area and Circle.area
// ✅ Searching "test_" lists all test functions
// ✅ Searching "Direction" finds the enum
// ============================================================

// ============================================================
// FIND ALL REFERENCES
// ✅ Right-click "Direction" below → Find All References
//   should list: enum declaration + every usage site in this file
// ✅ Right-click "greet" → Find All References
//   lists declaration + calls in inlay_hints_demo + main
// ============================================================

fn references_demo() {
	d := Direction.north
	if d == Direction.south {
		println('south')
	}
	_ := greet('references test')
}

// ============================================================
// RENAME SYMBOL
// ✅ Right-click "rename_me" below → Rename Symbol → type new name
//   Both declaration and usage sites update atomically
// ============================================================

fn rename_me() string {
	return 'original name'
}

fn rename_usage_demo() {
	_ := rename_me()
}

// ============================================================
// GO-TO-DEFINITION
// ✅ Ctrl/Cmd+click (or F12) on each identifier → jumps to declaration:
//   Point, Rectangle, Circle, Shape, Direction, greet, safe_divide,
//   find_item, total, math.sqrt, os.args, str.join
// ============================================================

fn goto_definition_demo() {
	p := Point{
		x: 1.0
		y: 2.0
	}
	r := Rectangle{
		width:  5.0
		height: 3.0
	}
	c := Circle{
		center: p
		radius: 2.0
	}
	d := Direction.east
	_ = r.area() // → Rectangle.area()
	_ = c.perimeter() // → Circle.perimeter()
	_ = p.distance_to(Point{ x: 4.0, y: 6.0 })
	_ = math.sqrt(9.0) // → stdlib
	_ = os.args // → stdlib
	_ = str.join(['x', 'y'], ',') // → stdlib (alias)
	_ = d
}

// ============================================================
// GO-TO-TYPE-DEFINITION
// ✅ Right-click any variable below → Go to Type Definition
//   p → jumps to struct Point
//   d → jumps to enum Direction
//   s → jumps to interface Shape
// ============================================================

fn goto_type_demo() {
	p := Point{
		x: 0.0
		y: 0.0
	}
	d := Direction.west
	s := Shape(Rectangle{
		width:  2.0
		height: 2.0
	})
	_ = p
	_ = d
	_ = s
}

// ============================================================
// GO-TO-IMPLEMENTATION
// ✅ Right-click "Shape" in the interface declaration → Go to Implementation
//   → lists Rectangle.area, Circle.area, Rectangle.perimeter, Circle.perimeter
// ✅ Right-click "Rectangle" struct → Go to Implementation
//   → lists Shape, Drawable as interfaces it satisfies
// ============================================================

// ============================================================
// DOCUMENT HIGHLIGHTS
// ✅ Click on "factor" in scale() — all three occurrences highlight
// ✅ Click on "width" — read sites and write sites highlighted differently
// ✅ Click on "Direction" — declaration + all enum fetch usages highlight
// ============================================================

// ============================================================
// CODE ACTIONS
// ✅ "Make Mutable" — uncomment the block in DIAGNOSTICS TRIGGERS above
// ✅ "Make Public" — right-click any `fn` or `struct` without `pub`
// ✅ "Add [heap]" — right-click any struct name
// ✅ "Add [flag]" — right-click any enum name without [flag]
// ✅ "Import Module" — trigger undefined ident error (see DIAGNOSTICS above)
// ============================================================

// ============================================================
// FORMATTING  (v fmt)
// ✅ Deliberately mis-indented code below — press the format shortcut.
//   After formatting, spacing and alignment should be corrected.
// ============================================================

fn formatting_demo() {
	x := 1 + 2
	y := x * 3
	if x > 0 {
		println('positive')
	}
	arr := [1, 2, 3]
	_ = y
	_ = arr
}

// ============================================================
// CONCURRENCY — spawn, channels, lock, shared
// ✅ Hover on "ch" shows inferred channel type
// ✅ Code lens "▶ Run" appears above the spawned fn literal
// ============================================================

fn concurrency_demo() {
	ch := chan int{cap: 2}

	spawn fn () {
		ch <- 10
		ch <- 20
	}()

	a := <-ch
	b := <-ch
	println(a + b)
	ch.close()
}

// ============================================================
// ENUMS — fetch, match, bitfield
// ✅ Hover on Direction.north → shows enum + field doc
// ✅ Inlay hints show implicit numeric values on Direction fields
// ✅ Permission is a [flag] enum — bitwise OR is valid
// ============================================================

fn enum_demo() {
	d := Direction.north
	match d {
		.north { println('N') }
		.south { println('S') }
		.east { println('E') }
		.west { println('W') }
	}

	perms := Permission.read | Permission.write
	if perms.has(.read) {
		println('can read')
	}

	s := Status.ok
	println(int(s))
}

// ============================================================
// ERROR HANDLING — !, ?, or {}, if unwrap
// ✅ Inlay hint "err →" shown inside each or {} block
// ✅ Hover on `err` inside or block shows its type (IError)
// ============================================================

fn error_handling_demo() {
	// or block
	q := safe_divide(9.0, 3.0) or {
		eprintln('divide failed: ${err}')
		0.0
	}

	// if unwrap
	if val := safe_divide(4.0, 2.0) {
		println('got ${val}')
	} else {
		println('error: ${err}')
	}

	// option
	item := find_item(['apple', 'banana'], 'banana') or { 'not found' }
	println(item)

	_ = q
}

// ============================================================
// COMPILE-TIME CONSTRUCTS
// ✅ $if / $else highlighted as keywords
// ✅ Hover on @FILE shows compile-time constant description
// ============================================================

fn compile_time_demo() {
	$if windows {
		println('Windows build')
	} $else {
		println('non-Windows build')
	}

	println(@FILE)
	println(@LINE.str())
	println(@MOD)
	println(@FN)
}

// ============================================================
// UNSAFE & ASM
// ✅ "unsafe" keyword highlighted; block is foldable
// ✅ Hover inside unsafe block still provides type information
// ============================================================

fn unsafe_demo() {
	unsafe {
		a := 42
		p := &a
		println(*p)
	}
}

// ============================================================
// SQL ORM
// ✅ "sql" keyword highlighted
// (Full ORM requires a DB connection; this shows syntax coverage only)
// ============================================================

// struct User {
// 	id   int    [primary; serial]
// 	name string [nonull]
// }
//
// fn sql_demo(db sqlite.DB) {
// 	users := sql db {
// 		select from User where id > 0
// 	}
// 	_ = users
// }
// ============================================================
// SMART AUTO-CLOSING
// ✅ Type { and cursor should be placed inside with } auto-inserted
// ✅ Same for [ ] ( ) " " ' ' ` `
// (Test by typing in an empty fn body)
// ============================================================

// ============================================================
// RAINBOW BRACKETS
// ✅ Deeply nested brackets below should each get a distinct colour
//   (requires "colorize_brackets": true in settings.json)
// ============================================================

fn rainbow_brackets_demo() {
	result := [[1, 2], [3, [4, 5]]]
	nested := (((1 + 2) * (3 + 4)) - (5 * (6 - 1)))
	_ = result
	_ = nested
}

// ============================================================
// BLOCK COMMENT TOGGLE
// ✅ Select lines below, press Ctrl+/ — they should become // comments
// ✅ Press Ctrl+/ again — comments should be removed
// ============================================================

fn comment_toggle_demo() {
	a := 1
	b := 2
	c := a + b
	_ = c
}

// ============================================================
// SNIPPETS — each prefix listed in the README
// ✅ Type each prefix in an empty context and press Tab.
//   Verify the expanded template appears with correct placeholders.
//
//   fn        fnpub      fnr        fnresult   fnoption
//   method    methodpub  methodmut
//   struct    structpub  interface  enum       typealias  sumtype
//   if        ifelse     iferr      match
//   forrange  forin      forindex   forc
//   orblock   orpanic    orreterr
//   defer     spawn      chan       lock
//   const     module     import     importas
//   test      assert     assertmsg
//   println   print      dump       eprintln
//   structlit array      map        interp     unsafe
//   sql       route      header
// ============================================================

// ============================================================
// WORD SELECTION
// ✅ Double-click on "snake_case_identifier" below — the full identifier
//   including underscores should be selected, not just one word segment.
// ============================================================

fn word_selection_demo() {
	snake_case_identifier := 'full selection test'
	another_long_variable_name := 42
	_ = snake_case_identifier
	_ = another_long_variable_name
}

// ============================================================
// HOVER — rich markdown for every symbol kind
// ✅ Hover over each identifier kind listed here:
// ============================================================

fn hover_demo() {
	// Function
	_ := greet('hover')

	// Method
	r := Rectangle{
		width:  3.0
		height: 4.0
	}
	_ = r.area()

	// Struct
	_ := Rectangle{}

	// Interface (used as type)
	shapes := []Shape{}
	_ = shapes

	// Enum + field
	d := Direction.north
	_ = d

	// Constant
	_ = max_items

	// Type alias
	m := Meters(100.0)
	_ = m

	// Variable with inferred type (hover shows inferred type)
	inferred := largest(10, 20)
	_ = inferred

	// Parameter (hover inside fn body on param name)
	_ := fn (val int) int {
		return val * 2
	}(5)

	// Keyword: hover over "chan" keyword in a channel type
	ch := chan string{cap: 1}
	ch.close()

	// Import path (hover over "math" in math.sqrt)
	_ = math.sqrt(16.0)

	// Stdlib function
	_ = os.getenv('HOME')

	// String interpolation parts
	name := 'world'
	_ = 'Hello, ${name}!'
}

// ============================================================
// SEMANTIC TOKENS
// ✅ Verify colouring distinguishes:
//   - User-defined fn (greet) vs builtin fn (println, len)
//   - Struct name (Rectangle) vs primitive type (int, f64)
//   - Interface name (Shape) vs enum name (Direction)
//   - Read access vs write access on mut variables (see scale() above)
// ============================================================

fn semantic_tokens_demo() {
	println('builtin') // builtin function
	_ := greet('user-defined') // user function — should differ in colour
	_ := Rectangle{} // struct type
	_ := Direction.north // enum type
	x := [1, 2, 3]
	_ = x.len // builtin property
}

// ============================================================
// MAIN
// ✅ Code lens shows "▶ Run workspace" and "▶ single file" above this fn
// ============================================================

fn main() {
	println(greet('V Enhanced'))

	inlay_hints_demo()
	folding_demo()
	references_demo()
	rename_usage_demo()
	goto_definition_demo()
	goto_type_demo()
	enum_demo()
	error_handling_demo()
	compile_time_demo()
	unsafe_demo()
	concurrency_demo()
	rainbow_brackets_demo()
	comment_toggle_demo()
	word_selection_demo()
	hover_demo()
	semantic_tokens_demo()
	formatting_demo()

	// multi-return
	n, s, ok := multi_return()
	println('${n} ${s} ${ok}')

	// variadic
	println(total(1, 2, 3, 4, 5))

	// generic
	println(largest(3.14, 2.71))

	// stdlib via alias
	println(str.join(['v', 'enhanced'], '-'))

	// dump() — shows name, type, and value in the console
	dump(max_items)
	dump(version)
}
