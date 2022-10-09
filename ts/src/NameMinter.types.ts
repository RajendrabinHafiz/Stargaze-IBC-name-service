/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.16.5.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export interface CountResponse {
  count: number;
  [k: string]: unknown;
}
export type ExecuteMsg = {
  increment: {
    [k: string]: unknown;
  };
} | {
  reset: {
    count: number;
    [k: string]: unknown;
  };
};
export interface InstantiateMsg {
  count: number;
  [k: string]: unknown;
}
export type QueryMsg = {
  get_count: {
    [k: string]: unknown;
  };
};
export type Addr = string;
export interface State {
  count: number;
  owner: Addr;
  [k: string]: unknown;
}