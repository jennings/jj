// Copyright 2020 The Jujutsu Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::io::{self, Read};

use starlark::environment::{Globals, GlobalsBuilder, Module};
use starlark::eval::Evaluator;
use starlark::syntax::{AstModule, Dialect};
use starlark::starlark_module;
use starlark_syntax::Error;
use serde::{Serialize, Deserialize};
use tracing::{instrument, Instrument};

use crate::cli_util::{CommandHelper, WorkspaceCommandHelper, WorkspaceCommandTransaction};
use crate::command_error::{CommandError, CommandErrorKind};
use crate::ui::Ui;

#[derive(clap::Args, Clone, Debug)]
pub(crate) struct ScriptArgs {
    /// The script to execute. Omit to read the script from stdin.
    #[arg()]
    script: Option<String>,
}

#[instrument(skip_all)]
pub(crate) fn cmd_script(
    ui: &mut Ui,
    command: &CommandHelper,
    args: &ScriptArgs,
) -> Result<(), CommandError> {
    let ast = AstModule::parse("script.star", args.script.unwrap(), &Dialect::Standard)
        .map_err(|e| CommandError::new(CommandErrorKind::User, e))?;
    let globals = GlobalsBuilder::new().with(jj).build();
    let module = Module::new();
    let mut eval = Evaluator::new(&module);
    let res = eval.eval_module(ast, &globals)
        .map_err(|e| CommandError::new(CommandErrorKind::User, e))?;
    Ok(())
}

#[starlark_module]
fn jj(builder: &mut GlobalsBuilder) {
    fn println(s: String) -> anyhow::Result<()> {
        println!("{}", s);
        Ok(())
    }
}
