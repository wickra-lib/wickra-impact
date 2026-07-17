/* R .Call glue for the wickra-impact C ABI hub. */
#include <R.h>
#include <Rinternals.h>
#include <R_ext/Rdynload.h>
#include <stddef.h>
#include "wickra_impact.h"

/* --- handle lifetime ----------------------------------------------------- */

static void wkimpact_finalize(SEXP ext) {
    WickraImpact *h = (WickraImpact *)R_ExternalPtrAddr(ext);
    if (h) {
        wickra_impact_free(h);
    }
    R_ClearExternalPtr(ext);
}

static WickraImpact *handle_of(SEXP ext) {
    WickraImpact *h = (WickraImpact *)R_ExternalPtrAddr(ext);
    if (!h) {
        Rf_error("wickra-impact: handle is closed");
    }
    return h;
}

/* --- exported .Call entries ---------------------------------------------- */

SEXP wkimpact_version(void) {
    return Rf_mkString(wickra_impact_version());
}

SEXP wkimpact_new(SEXP spec_json) {
    const char *spec = CHAR(STRING_ELT(spec_json, 0));
    WickraImpact *h = wickra_impact_new(spec);
    if (!h) {
        Rf_error("wickra-impact: invalid spec");
    }
    SEXP ext = PROTECT(R_MakeExternalPtr(h, R_NilValue, R_NilValue));
    R_RegisterCFinalizerEx(ext, wkimpact_finalize, TRUE);
    UNPROTECT(1);
    return ext;
}

SEXP wkimpact_command(SEXP ext, SEXP cmd_json) {
    WickraImpact *h = handle_of(ext);
    const char *cmd = CHAR(STRING_ELT(cmd_json, 0));

    /* Length-out protocol: learn the length, then read into a caller buffer.
       Domain errors come back in-band as {"ok":false,...} JSON, not a negative
       code; only unusable arguments / a caught panic return < 0. */
    int len = wickra_impact_command(h, cmd, NULL, 0);
    if (len < 0) {
        Rf_error("wickra-impact: command failed (code %d)", len);
    }
    char *buf = (char *)R_alloc((size_t)len + 1, 1);
    wickra_impact_command(h, cmd, buf, (size_t)len + 1);
    return Rf_mkString(buf);
}

/* --- registration -------------------------------------------------------- */

static const R_CallMethodDef CallEntries[] = {
    {"wkimpact_version", (DL_FUNC)&wkimpact_version, 0},
    {"wkimpact_new", (DL_FUNC)&wkimpact_new, 1},
    {"wkimpact_command", (DL_FUNC)&wkimpact_command, 2},
    {NULL, NULL, 0}};

void R_init_wickraimpact(DllInfo *dll) {
    R_registerRoutines(dll, NULL, CallEntries, NULL, NULL);
    R_useDynamicSymbols(dll, FALSE);
}
