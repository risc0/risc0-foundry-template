#!/usr/bin/env node

import { subcommands, run } from "cmd-ts";
import { deploy } from "./deploy";
import { submit } from "./submit";


const root = subcommands({
  name: "even-number",
  cmds: {
    deploy,
    submit: submit
  },
});

process.on("unhandledRejection", (reason, promise) => {
  console.log(
    "[unhandledRejection] Unhandled rejection at ",
    promise,
    `reason: ${reason}`
  );
  process.exit(1);
});

process.on("uncaughtException", (err) => {
  console.log(`[uncaughtException] Uncaught Exception: ${err.message}`);
  process.exit(1);
});

run(root, process.argv.slice(2))
  .then(() => {
    process.exit();
  })
  .catch((error) => {
    console.error("even-number error: ", error);
    process.exit(1);
  });
