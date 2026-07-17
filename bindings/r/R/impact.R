#' The wickra-impact library version.
#' @return A version string.
#' @export
wkimpact_version <- function() {
  .Call(C_wkimpact_version)
}

#' Build a backtest handle from a spec JSON.
#' @param spec_json An `ImpactSpec` JSON string (`"{}"` defers configuration to a
#'   later `set_spec` command).
#' @return A `wickra_impact` handle (an external pointer).
#' @export
wkimpact_new <- function(spec_json) {
  .Call(C_wkimpact_new, spec_json)
}

#' Apply a command JSON and return the resulting response JSON.
#' @param impact A backtest handle from [wkimpact_new()].
#' @param cmd_json A command JSON string (`set_spec`, `run`, `version`).
#' @return The response as a JSON string.
#' @export
wkimpact_command <- function(impact, cmd_json) {
  .Call(C_wkimpact_command, impact, cmd_json)
}
