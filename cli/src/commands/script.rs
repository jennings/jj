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

use mlua::prelude::{Lua, LuaResult, FromLua, IntoLua};
use serde::{Serialize, Deserialize};
use tracing::instrument;

use crate::cli_util::{CommandHelper, WorkspaceCommandHelper, WorkspaceCommandTransaction};
use crate::command_error::CommandError;
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
    let lua = Lua::new();
    
    let map_table = lua.create_table().unwrap();
    map_table.set("println", lua.create_function(println).unwrap()).unwrap();

    lua.globals().set("jj", map_table).unwrap();

    let script = if let Some(script) = &args.script {
        script.clone()
    } else {
        let mut stdin = std::io::stdin();
        let mut script = String::new();
        stdin.read_to_string(&mut script).unwrap();
        script
    };
    lua.load(script).exec().unwrap();

    Ok(())
}

fn println(_: &Lua, value: String) -> LuaResult<()> {
    println!("{}", value);
    Ok(())
}

/*
impl<'a> IntoLua for WorkspaceCommandTransaction<'a> {
    fn into_lua(self, lua: &Lua) -> LuaResult<mlua::Value> {
        let table = lua.create_table()?;
        table.set("finish", |_, description: String| self.finish(ui, description))?;
        table
    }
}

impl IntoLua for WorkspaceCommandHelper {
    fn into_lua(self, lua: &Lua) -> LuaResult<mlua::Value> {
        let table = lua.create_table()?;
        table.set("start_transaction", |_, _| self.start_transaction())?;
        table
    }
}

impl FromLua for WorkspaceCommandHelper {
    fn from_lua(value: mlua::Value, lua: &Lua) -> LuaResult<Self> {
        todo!()
    }
}

fn start_transaction(_: &Lua, name: String) -> LuaResult<()> {
    Ok(())
}
*/
