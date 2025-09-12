import { exit } from "process";
import { VoteProof } from "./molecules";
import { ccc, Transaction, WitnessArgs, OutPoint} from "@ckb-ccc/core";

async function main() {
  const cccClient = new ccc.ClientPublicTestnet();
  const signer = new ccc.SignerCkbPrivateKey(cccClient, '0x88179b7e387921a193f459859d8ff461e973a93a449b4930179928dbc53a04ba');

  const voteAddress =
    "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqwu8lmjcalepgp5k6d4j0mtxwww68v9m6qz0q8ah";

  const voteAddr = await ccc.Address.fromString(voteAddress, cccClient);

  // vote "good"
  const candidates = ["no", "not bad", "good", "awesome"]
  const voteIndex = candidates.indexOf("good");
  const voteNum = 1 << (voteIndex + 1); 
  // voteData is 32 bit little endian
  const voteData = voteNum.toString(16).padStart(8, "0");
  console.log("vote data: ", ccc.hexFrom(voteData));

  // user smt proof from smt
  const userSmtProof = "0x4c4ffc51fc460c9bae4a5afce6de58192cc0a7416b9a62627878627013d983c76ef3b3485d1380b63765aacc617854fd83434a90caa535ba7117fbf3d9d01ba7f1d472b50151fdf5df9651f803196883b8904d1cd5d38ccac9e658071f7882d45888b3bfb0cf357c9b0e612253e09f74501e84982cedbf099ee836f158de22d4a294c9e43a290b51fe504a34e98c2c3101dac192d1046c6d7d1a365de76159803ede966affdae1c160e20182d70b2c498483d626f9f558050a8e0df731bbafdfa277e34e3a0a7450364f01";
  // build vote proof
  const voteProof = new VoteProof(
    voteAddr.script.hash(),
    userSmtProof,
  )
  const voteProofBytes = voteProof.toBytes();
  const voteProofHex = ccc.hexFrom(voteProofBytes);
  console.log("vote proof: ", voteProofHex);

  // cell deps of vote meta
  const voteMetaOutpoint = {
    txHash: "0x4cea81eaaf5ae7086f73d0b42d2295aa9103057c880a5b9f54849422aafb5297",
    index: 0,
  }
  const voteMetaCellDep = {
    outPoint: voteMetaOutpoint,
    depType: "code",
  }

  // typescript of vote contract
  // TODO: replace with real vote contract type script and add cell dep of vote contract later
  // args is blake160 hash of vote meta cell out point
  const voteTypeArgs = OutPoint.from(voteMetaOutpoint).hash().slice(0, 42);
  console.log("vote type args: ", voteTypeArgs);
  const voteTypeScript = {
    codeHash: "0x00000000000000000000000000000000000000000000000000545950455f4944",
    args: voteTypeArgs,
    hashType: "type",
  }

  // create a vote cell
  const tx = Transaction.from({
    cellDeps: [
      voteMetaCellDep,
    ],
    outputs: [
      {
        lock: voteAddr.script,
        type: voteTypeScript,
      }
    ],
    outputsData: [voteData],
  })

  await tx.completeInputsAtLeastOne(signer);
  await tx.completeFeeBy(signer);

  // set vote proof into witness
  let witnessArgs = WitnessArgs.from({
    outputType: voteProofBytes,
  });
  tx.setWitnessArgsAt(0, witnessArgs);

  await signer.signTransaction(tx);

  console.log("tx: ", ccc.stringify(tx));
  console.log("tx hash: ", tx.hash());

  // just for test no need send tx
  //const txHash = await signer.sendTransaction(tx);
  //console.log("The transaction hash is", txHash);

  // destroy vote cell to get back ckb

  // for test we haven't send vote tx yet, so no need destroy
  // const destroyTx = Transaction.from({
  //   inputs: [
  //     {
  //       previousOutput: {
  //         txHash: tx.hash(),
  //         index: 0,
  //       },
  //       since: 0,
  //     }
  //   ],
  //   outputs: [
  //     {
  //       lock: voteAddr.script,
  //       type: null,
  //     }
  //   ],
  //   outputsData: ["0x"],
  // })
  // await destroyTx.completeInputsByCapacity(signer);
  // await destroyTx.completeFeeBy(signer);
  // await signer.signTransaction(destroyTx);
  // console.log("destroy tx: ", ccc.stringify(destroyTx));
  // console.log("destroy tx hash: ", destroyTx.hash());
}

main().then(() => process.exit());
