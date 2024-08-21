import { options, config } from "@nebrazkp/upa/tool";
const { keyfile, endpoint, password } = options;
const { loadWallet, } = config;
import * as ethers from "ethers";
import { command, number, option, positional, string } from "cmd-ts";
import { EvenNumber__factory } from "../typechain-types";
import { EvenNumberInstance } from "./deploy";
import * as fs from "fs";

export const submit = command({
  name: "submit",
  args: {
    endpoint: endpoint(),
    keyfile: keyfile(),
    password: password(),
    evenNumberInstanceFile: option({
        type: string,
        long: "instance",
        defaultValue: () => "even-number.instance",
        description: "even-number instance file",
      }),
      newNumber: positional({
        type: number,
        description: "Set the stored even number to this number",
      })
  },
  description:
    "Set a new even number, whose even-ness was proven in the UPA contract",
  handler: async function ({
    endpoint,
    keyfile,
    password,
    evenNumberInstanceFile,
    newNumber
  }): Promise<void> {
    const provider = new ethers.JsonRpcProvider(endpoint);
    const wallet = await loadWallet(keyfile, password, provider);

    const evenNumberInstance = JSON.parse(fs.readFileSync(evenNumberInstanceFile, "ascii")) as EvenNumberInstance
    const evenNumber = EvenNumber__factory.connect(evenNumberInstance.evenNumber).connect(
      wallet
    );
    
    const txResponse = await evenNumber.set(newNumber);
    await txResponse.wait();
    
    console.log(`Even number successfully set to ${newNumber}`);
  },
});
