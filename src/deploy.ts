import * as ethers from "ethers";
import { command, option, string } from "cmd-ts";
import * as fs from "fs";
import { EvenNumber__factory } from "../typechain-types";
import { options, config } from "@nebrazkp/upa/tool";
const { keyfile, endpoint, password } = options;
const { loadWallet, upaFromInstanceFile } = config;

export type EvenNumberInstance = {
    /// Address of the even-number contract
    evenNumber: string;
  };
  
export const deploy = command({
  name: "deploy",
  args: {
    endpoint: endpoint(),
    keyfile: keyfile(),
    password: password(),
    upaInstance: option({
        type: string,
        long: "upa-instance",
        defaultValue: () => `upa.instance`,
        description: "The UPA instance used to deploy",
      }),
  },
  description: "Deploy the even number contract.",
  handler: async function ({
    endpoint,
    keyfile,
    password,
    upaInstance,
  }): Promise<void> {
    const provider = new ethers.JsonRpcProvider(endpoint);
    const wallet = await loadWallet(keyfile, password, provider);

    const upa = await upaFromInstanceFile(upaInstance, provider);

    const EvenNumber = new EvenNumber__factory(wallet);
    const evenNumber = await EvenNumber.deploy(upa.verifier);
    await evenNumber.waitForDeployment();


    // Write the instance information to disk
    const instanceData: EvenNumberInstance = {
      evenNumber: await evenNumber.getAddress(),
    };
    fs.writeFileSync("even-number.instance", JSON.stringify(instanceData));

    console.log(`EvenNumber was deployed to address \
    ${instanceData.evenNumber}`);
  },
});
