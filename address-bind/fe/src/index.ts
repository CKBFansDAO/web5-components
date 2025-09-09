import { BindInfo } from "./molecules";
import { ccc } from "@ckb-ccc/core";

async function main() {
  const cccClient = new ccc.ClientPublicTestnet();

  const fromAddress =
    "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsq2jk6pyw9vlnfakx7vp4t5lxg0lzvvsp3c5adflu";

  const fromAddr = await ccc.Address.fromString(fromAddress, cccClient);

  const toAddress =
    "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsq2jk6pyw9vlnfakx7vp4t5lxg0lzvvsp3c5adflu";

  const toAddr = await ccc.Address.fromString(toAddress, cccClient);

  const bindInfo = BindInfo.from({
    from: fromAddr.script,
    to: toAddr.script,
    timestamp: BigInt(Date.now())
  });

  const bindInfoBytes = bindInfo.toBytes();

  console.log("bind info: ", bindInfoBytes);
}

main();