// Command discarded prints, for every value a Go module's own packages assign to the blank
// identifier, where it is, its type and whether that type implements error -- one JSON object per
// line.
//
// It is nomos-lang-go-types' helper, run by the Go toolchain on the host from this one file. The
// standard library only, so running it needs no module download: package metadata and export data
// come from `go list -e -test -export -compiled -deps -json ./...`, each package's compiled files
// are parsed with go/parser, and go/types checks each package against its dependencies' export
// data. A package is checked once, as its test variant when it has one, so a test file is judged
// and a non-test file is not judged twice.
//
// Three kinds of line, each a JSON object with a "kind":
//
//	{"kind":"checked","file":F}                                          a file of a package that type-checked
//	{"kind":"value","file":F,"line":L,"column":C,"type":T,"is_error":B}  one value assigned to _
//	{"kind":"failed","package":P,"files":[F...],"reason":R}              a package that did not load or type-check
//
// Every file is an absolute path, and every position is the file's own -- a //line directive does
// not move it, so a cgo source, which is compiled from a generated file, is never named checked.
//
// Its arguments are the go program to run `go list` with, so the helper asks the same toolchain
// that ran it, and the module directory to run it in. Exit 0 once `go list` has answered, whatever
// it answered about each package; exit 1, with the reason on stderr, when it did not.
//
// Every function is exported, and named in Upper_Snake_Case, because that is the one spelling both
// of this repository's function-name conventions accept; in a main package, exporting a name
// changes nothing else.
package main

import (
	"bufio"
	"bytes"
	"encoding/json"
	"fmt"
	"go/ast"
	"go/importer"
	"go/parser"
	"go/token"
	"go/types"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

// listed is one package as `go list -json` describes it, read field by field so that each field
// keeps a lower_snake name here.
type listed struct {
	import_path       string
	for_test          string
	directory         string
	go_files          []string
	cgo_files         []string
	compiled_go_files []string
	export            string
	import_map        map[string]string
	in_main_module    bool
	failure           string
}

// blank is one value assigned to the blank identifier.
type blank struct {
	position   token.Pos
	value_type types.Type
}

var error_interface = types.Universe.Lookup("error").Type().Underlying().(*types.Interface)

func main() {
	if len(os.Args) != 3 {
		fmt.Fprintln(os.Stderr, "usage: discarded <go program> <module directory>")
		os.Exit(2)
	}
	packages, err := List(os.Args[1], os.Args[2])
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	exports := map[string]string{}
	test_variants := map[string]bool{}
	for _, listed_package := range packages {
		if listed_package.export != "" {
			exports[listed_package.import_path] = listed_package.export
		}
		if listed_package.for_test != "" && strings.HasPrefix(listed_package.import_path, listed_package.for_test+" [") {
			test_variants[listed_package.for_test] = true
		}
	}
	out := bufio.NewWriter(os.Stdout)
	for _, listed_package := range packages {
		if Judged(listed_package, test_variants) {
			Answer(out, listed_package, exports)
		}
	}
	if err := out.Flush(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}

// Judged says whether p is a package of the main module to check: its test variant when it has
// one, the package itself otherwise, each external test package, and never a generated test main.
func Judged(p listed, test_variants map[string]bool) bool {
	if !p.in_main_module {
		return false
	}
	if p.for_test != "" {
		return true
	}
	if strings.HasSuffix(p.import_path, ".test") {
		return false
	}
	return !test_variants[p.import_path]
}

// Answer writes p's lines: its checked files and every value it discards, or why it failed. A
// package with nothing to compile -- every file excluded by a build constraint -- writes nothing,
// so its files are reported by the caller as unchecked rather than as a package that failed.
func Answer(out *bufio.Writer, p listed, exports map[string]string) {
	compiled := Absolute(p.directory, p.compiled_go_files)
	if len(compiled) == 0 {
		return
	}
	failed := func(reason string) {
		sources := Absolute(p.directory, append(append([]string{}, p.go_files...), p.cgo_files...))
		Write(out, map[string]any{"kind": "failed", "package": p.import_path, "files": sources, "reason": strings.TrimSpace(reason)})
	}
	if p.failure != "" {
		failed(p.failure)
		return
	}
	positions := token.NewFileSet()
	var files []*ast.File
	for _, path := range compiled {
		file, err := parser.ParseFile(positions, path, nil, 0)
		if err != nil {
			failed(err.Error())
			return
		}
		files = append(files, file)
	}
	lookup := func(path string) (io.ReadCloser, error) {
		if mapped, ok := p.import_map[path]; ok {
			path = mapped
		}
		export, ok := exports[path]
		if !ok {
			return nil, fmt.Errorf("no export data for %s, which did not compile", path)
		}
		return os.Open(export)
	}
	info := &types.Info{Types: map[ast.Expr]types.TypeAndValue{}, Defs: map[*ast.Ident]types.Object{}}
	config := types.Config{Importer: importer.ForCompiler(positions, "gc", lookup)}
	if _, err := config.Check(p.import_path, positions, files, info); err != nil {
		failed(err.Error())
		return
	}
	for _, path := range compiled {
		Write(out, map[string]any{"kind": "checked", "file": path})
	}
	for _, file := range files {
		ast.Inspect(file, func(node ast.Node) bool {
			for _, value := range Discarded(node, info) {
				at := positions.PositionFor(value.position, false)
				is_error := types.Implements(value.value_type, error_interface)
				Write(out, map[string]any{"kind": "value", "file": at.Filename, "line": at.Line, "column": at.Column, "type": value.value_type.String(), "is_error": is_error})
			}
			return true
		})
	}
}

// Discarded is every value node assigns to the blank identifier, with its type: an assignment or
// short declaration, a var or const declaration's initializer -- written, or in a const block
// repeated from the line above -- or a range clause. A var declared with no initializer is sent no
// value, so it is not one.
func Discarded(node ast.Node, info *types.Info) []blank {
	switch statement := node.(type) {
	case *ast.AssignStmt:
		return Assigned(statement.Lhs, statement.Rhs, info)
	case *ast.ValueSpec:
		if len(statement.Values) == 0 && statement.Type == nil {
			return Repeated(statement, info)
		}
		names := make([]ast.Expr, len(statement.Names))
		for i, name := range statement.Names {
			names[i] = name
		}
		return Assigned(names, statement.Values, info)
	case *ast.RangeStmt:
		return Ranged(statement, info)
	}
	return nil
}

// Assigned is each blank target of left, typed by the value right gives it -- one value per target,
// or one call or comma-ok expression whose results are a tuple.
func Assigned(left []ast.Expr, right []ast.Expr, info *types.Info) []blank {
	var values []blank
	for i, target := range left {
		if !Is_Blank(target) {
			continue
		}
		var value_type types.Type
		if len(right) == len(left) {
			value_type = info.Types[right[i]].Type
		} else if len(right) == 1 {
			if tuple, ok := info.Types[right[0]].Type.(*types.Tuple); ok && i < tuple.Len() {
				value_type = tuple.At(i).Type()
			}
		}
		if value_type != nil {
			values = append(values, blank{position: target.Pos(), value_type: value_type})
		}
	}
	return values
}

// Repeated is each blank name of a const spec that repeats the initializer above it -- the one
// spec with neither a value nor a type -- typed as the constant the checker declared for it.
func Repeated(statement *ast.ValueSpec, info *types.Info) []blank {
	var values []blank
	for _, name := range statement.Names {
		if declared := info.Defs[name]; declared != nil && Is_Blank(name) {
			values = append(values, blank{position: name.Pos(), value_type: declared.Type()})
		}
	}
	return values
}

// Ranged is the key and value of a range clause that go to the blank identifier, typed by what the
// clause ranges over.
func Ranged(statement *ast.RangeStmt, info *types.Info) []blank {
	over := info.Types[statement.X].Type
	if over == nil {
		return nil
	}
	key, element := Iteration_Types(Core_Type(over))
	var values []blank
	for _, target := range []struct {
		expression ast.Expr
		value_type types.Type
	}{{statement.Key, key}, {statement.Value, element}} {
		if target.expression != nil && Is_Blank(target.expression) && target.value_type != nil {
			values = append(values, blank{position: target.expression.Pos(), value_type: target.value_type})
		}
	}
	return values
}

// Core_Type is t's underlying type, or for a type parameter the underlying type of its
// constraint's first term -- which, since the range clause type-checked, every term shares.
func Core_Type(t types.Type) types.Type {
	parameter, ok := t.(*types.TypeParam)
	if !ok {
		return t.Underlying()
	}
	if term := First_Term(parameter.Constraint()); term != nil {
		return Core_Type(term)
	}
	return nil
}

// First_Term is the first type term a constraint lists, looking through the interfaces it embeds.
func First_Term(t types.Type) types.Type {
	switch constraint := t.Underlying().(type) {
	case *types.Interface:
		for i := 0; i < constraint.NumEmbeddeds(); i++ {
			if term := First_Term(constraint.EmbeddedType(i)); term != nil {
				return term
			}
		}
		return nil
	case *types.Union:
		if constraint.Len() > 0 {
			return constraint.Term(0).Type()
		}
		return nil
	}
	return t
}

// Iteration_Types is what a range clause over a value of underlying type t yields: its key and
// its element, either nil where the clause yields none.
func Iteration_Types(t types.Type) (types.Type, types.Type) {
	switch over := t.(type) {
	case *types.Basic:
		if over.Info()&types.IsString != 0 {
			return types.Typ[types.Int], types.Universe.Lookup("rune").Type()
		}
		return over, nil
	case *types.Array:
		return types.Typ[types.Int], over.Elem()
	case *types.Slice:
		return types.Typ[types.Int], over.Elem()
	case *types.Pointer:
		if array, ok := over.Elem().Underlying().(*types.Array); ok {
			return types.Typ[types.Int], array.Elem()
		}
	case *types.Map:
		return over.Key(), over.Elem()
	case *types.Chan:
		return over.Elem(), nil
	case *types.Signature:
		if over.Params().Len() == 1 {
			if yield, ok := over.Params().At(0).Type().Underlying().(*types.Signature); ok {
				var yielded [2]types.Type
				for i := 0; i < yield.Params().Len() && i < 2; i++ {
					yielded[i] = yield.Params().At(i).Type()
				}
				return yielded[0], yielded[1]
			}
		}
	}
	return nil, nil
}

// Is_Blank says whether expression is the blank identifier.
func Is_Blank(expression ast.Expr) bool {
	identifier, ok := expression.(*ast.Ident)
	return ok && identifier.Name == "_"
}

// Absolute is each of names as an absolute path, joined to directory when it is not one already.
func Absolute(directory string, names []string) []string {
	paths := make([]string, 0, len(names))
	for _, name := range names {
		if filepath.IsAbs(name) {
			paths = append(paths, name)
		} else {
			paths = append(paths, filepath.Join(directory, name))
		}
	}
	return paths
}

// Write writes one line: fields as one JSON object, its keys in sorted order.
func Write(out *bufio.Writer, fields map[string]any) {
	// Marshal fails only on a value JSON cannot hold, and every field here is text, a number, a
	// boolean or a list of text.
	encoded, _ := json.Marshal(fields)
	out.Write(encoded)
	out.WriteString("\n")
}

// List is every package `go list` names for the module in directory, with the dependencies and
// test variants it needs, as go_program answers.
func List(go_program string, directory string) ([]listed, error) {
	command := exec.Command(go_program, "list", "-e", "-test", "-export", "-compiled", "-deps", "-json", "./...")
	command.Dir = directory
	var stderr bytes.Buffer
	command.Stderr = &stderr
	output, err := command.Output()
	if err != nil {
		return nil, fmt.Errorf("go list: %v: %s", err, strings.TrimSpace(stderr.String()))
	}
	var packages []listed
	decoder := json.NewDecoder(bytes.NewReader(output))
	for decoder.More() {
		var fields map[string]json.RawMessage
		if err := decoder.Decode(&fields); err != nil {
			return nil, err
		}
		listed_package, err := Read_Package(fields)
		if err != nil {
			return nil, err
		}
		packages = append(packages, listed_package)
	}
	return packages, nil
}

// Read_Package reads the fields of one `go list -json` object this helper uses, each of which may
// be absent.
func Read_Package(fields map[string]json.RawMessage) (listed, error) {
	var p listed
	var module map[string]json.RawMessage
	var failure map[string]json.RawMessage
	if err := Read_Fields(fields, map[string]any{
		"ImportPath":      &p.import_path,
		"ForTest":         &p.for_test,
		"Dir":             &p.directory,
		"GoFiles":         &p.go_files,
		"CgoFiles":        &p.cgo_files,
		"CompiledGoFiles": &p.compiled_go_files,
		"Export":          &p.export,
		"ImportMap":       &p.import_map,
		"Module":          &module,
		"Error":           &failure,
	}); err != nil {
		return p, err
	}
	if err := Read_Fields(module, map[string]any{"Main": &p.in_main_module}); err != nil {
		return p, err
	}
	return p, Read_Fields(failure, map[string]any{"Err": &p.failure})
}

// Read_Fields decodes each field of fields that targets names into its target, leaving a target
// whose field is absent as it was.
func Read_Fields(fields map[string]json.RawMessage, targets map[string]any) error {
	for key, target := range targets {
		if raw, present := fields[key]; present {
			if err := json.Unmarshal(raw, target); err != nil {
				return fmt.Errorf("go list field %s: %v", key, err)
			}
		}
	}
	return nil
}
