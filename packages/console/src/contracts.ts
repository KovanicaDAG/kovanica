// Contract SDK typed wrappers for the console.
import { entryPoint, ContractKind, contractCall } from "@kovanica/api-client";
import type { KovanicaRpcClient } from "@kovanica/api-client";

export interface ContractCallParams {
  contract: string;
  method: string;
  args: number[];
  value?: number;
  gasLimit?: number;
}

export function buildContractCall(
  kind: ContractKind,
  method: string,
  args: number[] = [],
): ContractCallParams {
  const spec = contractCall(method, args);
  return {
    contract: kind,
    method: entryPoint(kind, spec.method),
    args: spec.args,
  };
}

export async function invokeContract(
  client: KovanicaRpcClient,
  params: ContractCallParams,
): Promise<unknown> {
  return client.call("kovanica_call", {
    contract: params.contract,
    method: params.method,
    args: params.args,
    value: params.value ?? 0,
    gas_limit: params.gasLimit ?? 1_000_000,
  });
}