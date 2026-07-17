// Package wickra provides idiomatic Go bindings for wickra-impact over its C ABI
// hub: build an Impact from a spec JSON, drive it with command JSON (set_spec,
// run, version) and read back the response JSON — the same protocol as the CLI
// and every other binding.
//
// The binding links the prebuilt C ABI library, staged per platform under
// ./lib/<goos>_<goarch>/, with the header vendored under ./include.
package wickra

/*
#cgo CFLAGS: -I${SRCDIR}/include
#cgo linux,amd64 LDFLAGS: -L${SRCDIR}/lib/linux_amd64 -lwickra_impact -Wl,-rpath,${SRCDIR}/lib/linux_amd64
#cgo linux,arm64 LDFLAGS: -L${SRCDIR}/lib/linux_arm64 -lwickra_impact -Wl,-rpath,${SRCDIR}/lib/linux_arm64
#cgo darwin,amd64 LDFLAGS: -L${SRCDIR}/lib/darwin_amd64 -lwickra_impact -Wl,-rpath,${SRCDIR}/lib/darwin_amd64
#cgo darwin,arm64 LDFLAGS: -L${SRCDIR}/lib/darwin_arm64 -lwickra_impact -Wl,-rpath,${SRCDIR}/lib/darwin_arm64
#cgo windows,amd64 LDFLAGS: -L${SRCDIR}/lib/windows_amd64 -l:wickra_impact.dll
#cgo windows,arm64 LDFLAGS: -L${SRCDIR}/lib/windows_arm64 -l:wickra_impact.dll
#include <stdlib.h>
#include "wickra_impact.h"
*/
import "C"

import (
	"fmt"
	"runtime"
	"unsafe"
)

// Impact is a market-impact backtest driven by JSON commands, built from a spec.
type Impact struct {
	handle *C.WickraImpact
}

// New builds a backtest handle from a spec JSON string ("{}" defers
// configuration to a later set_spec command). It returns an error if the spec is
// null, not valid UTF-8, or not a valid spec. Call Close when done (a finalizer
// also frees it, but explicit Close is preferred).
func New(specJSON string) (*Impact, error) {
	cspec := C.CString(specJSON)
	defer C.free(unsafe.Pointer(cspec))

	handle := C.wickra_impact_new(cspec)
	if handle == nil {
		return nil, fmt.Errorf("wickra-impact: invalid spec")
	}
	i := &Impact{handle: handle}
	runtime.SetFinalizer(i, (*Impact).Close)
	return i, nil
}

// Command applies a command JSON and returns the response JSON. It uses the C
// ABI's length-out protocol: a first call learns the length, then the response
// is read into a caller-owned buffer.
func (i *Impact) Command(cmdJSON string) (string, error) {
	ccmd := C.CString(cmdJSON)
	defer C.free(unsafe.Pointer(ccmd))

	n := C.wickra_impact_command(i.handle, ccmd, nil, 0)
	if n < 0 {
		return "", fmt.Errorf("wickra-impact: command failed (code %d)", int(n))
	}
	buf := make([]byte, int(n)+1)
	C.wickra_impact_command(
		i.handle,
		ccmd,
		(*C.char)(unsafe.Pointer(&buf[0])),
		C.uintptr_t(len(buf)),
	)
	return string(buf[:n]), nil
}

// Close frees the backtest handle. Safe to call more than once.
func (i *Impact) Close() {
	if i.handle != nil {
		C.wickra_impact_free(i.handle)
		i.handle = nil
	}
	runtime.SetFinalizer(i, nil)
}

// Version returns the library version.
func Version() string {
	return C.GoString(C.wickra_impact_version())
}
