import {
  mol,
  HexLike,
  hexFrom,
  Hex,
  NumLike,
  numFrom,
  Num,
  Script,
  ScriptLike,
} from "@ckb-ccc/core";


// table VoteProof {
//     lock_script_hash: Bytes,
//     smt_proof: Bytes,
// }

export type VoteProofLike = {
  lock_script_hash: HexLike;
  smt_proof: HexLike;
};

@mol.codec(
  mol.table({
    lock_script_hash: mol.Bytes,
    smt_proof: mol.Bytes,
  }),
)
export class VoteProof extends mol.Entity.Base<VoteProofLike, VoteProof>() {
  constructor(
    public lock_script_hash: HexLike,
    public smt_proof: HexLike,
  ) {
    super();
  }

  static from(data: VoteProofLike): VoteProof {
    if (data instanceof VoteProof) {
      return data;
    }
    return new VoteProof(
      hexFrom(data.lock_script_hash),
      hexFrom(data.smt_proof),
    );
  }
}
