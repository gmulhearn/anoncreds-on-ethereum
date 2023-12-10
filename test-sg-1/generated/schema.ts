// THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.

import {
  TypedMap,
  Entity,
  Value,
  ValueKind,
  store,
  Bytes,
  BigInt,
  BigDecimal
} from "@graphprotocol/graph-ts";

export class NewResourceEvent extends Entity {
  constructor(id: Bytes) {
    super();
    this.set("id", Value.fromBytes(id));
  }

  save(): void {
    let id = this.get("id");
    assert(id != null, "Cannot save NewResourceEvent entity without an ID");
    if (id) {
      assert(
        id.kind == ValueKind.BYTES,
        `Entities of type NewResourceEvent must have an ID of type Bytes but the id '${id.displayData()}' is of type ${id.displayKind()}`
      );
      store.set("NewResourceEvent", id.toBytes().toHexString(), this);
    }
  }

  static loadInBlock(id: Bytes): NewResourceEvent | null {
    return changetype<NewResourceEvent | null>(
      store.get_in_block("NewResourceEvent", id.toHexString())
    );
  }

  static load(id: Bytes): NewResourceEvent | null {
    return changetype<NewResourceEvent | null>(
      store.get("NewResourceEvent", id.toHexString())
    );
  }

  get id(): Bytes {
    let value = this.get("id");
    if (!value || value.kind == ValueKind.NULL) {
      throw new Error("Cannot return null for a required field.");
    } else {
      return value.toBytes();
    }
  }

  set id(value: Bytes) {
    this.set("id", Value.fromBytes(value));
  }

  get didIdentity(): Bytes {
    let value = this.get("didIdentity");
    if (!value || value.kind == ValueKind.NULL) {
      throw new Error("Cannot return null for a required field.");
    } else {
      return value.toBytes();
    }
  }

  set didIdentity(value: Bytes) {
    this.set("didIdentity", Value.fromBytes(value));
  }

  get path(): string {
    let value = this.get("path");
    if (!value || value.kind == ValueKind.NULL) {
      throw new Error("Cannot return null for a required field.");
    } else {
      return value.toString();
    }
  }

  set path(value: string) {
    this.set("path", Value.fromString(value));
  }

  get blockNumber(): BigInt {
    let value = this.get("blockNumber");
    if (!value || value.kind == ValueKind.NULL) {
      throw new Error("Cannot return null for a required field.");
    } else {
      return value.toBigInt();
    }
  }

  set blockNumber(value: BigInt) {
    this.set("blockNumber", Value.fromBigInt(value));
  }

  get blockTimestamp(): BigInt {
    let value = this.get("blockTimestamp");
    if (!value || value.kind == ValueKind.NULL) {
      throw new Error("Cannot return null for a required field.");
    } else {
      return value.toBigInt();
    }
  }

  set blockTimestamp(value: BigInt) {
    this.set("blockTimestamp", Value.fromBigInt(value));
  }

  get transactionHash(): Bytes {
    let value = this.get("transactionHash");
    if (!value || value.kind == ValueKind.NULL) {
      throw new Error("Cannot return null for a required field.");
    } else {
      return value.toBytes();
    }
  }

  set transactionHash(value: Bytes) {
    this.set("transactionHash", Value.fromBytes(value));
  }
}

export class StatusListUpdateEvent extends Entity {
  constructor(id: Bytes) {
    super();
    this.set("id", Value.fromBytes(id));
  }

  save(): void {
    let id = this.get("id");
    assert(
      id != null,
      "Cannot save StatusListUpdateEvent entity without an ID"
    );
    if (id) {
      assert(
        id.kind == ValueKind.BYTES,
        `Entities of type StatusListUpdateEvent must have an ID of type Bytes but the id '${id.displayData()}' is of type ${id.displayKind()}`
      );
      store.set("StatusListUpdateEvent", id.toBytes().toHexString(), this);
    }
  }

  static loadInBlock(id: Bytes): StatusListUpdateEvent | null {
    return changetype<StatusListUpdateEvent | null>(
      store.get_in_block("StatusListUpdateEvent", id.toHexString())
    );
  }

  static load(id: Bytes): StatusListUpdateEvent | null {
    return changetype<StatusListUpdateEvent | null>(
      store.get("StatusListUpdateEvent", id.toHexString())
    );
  }

  get id(): Bytes {
    let value = this.get("id");
    if (!value || value.kind == ValueKind.NULL) {
      throw new Error("Cannot return null for a required field.");
    } else {
      return value.toBytes();
    }
  }

  set id(value: Bytes) {
    this.set("id", Value.fromBytes(value));
  }

  get indexedRevocationRegistryId(): string {
    let value = this.get("indexedRevocationRegistryId");
    if (!value || value.kind == ValueKind.NULL) {
      throw new Error("Cannot return null for a required field.");
    } else {
      return value.toString();
    }
  }

  set indexedRevocationRegistryId(value: string) {
    this.set("indexedRevocationRegistryId", Value.fromString(value));
  }

  get revocationRegistryId(): string {
    let value = this.get("revocationRegistryId");
    if (!value || value.kind == ValueKind.NULL) {
      throw new Error("Cannot return null for a required field.");
    } else {
      return value.toString();
    }
  }

  set revocationRegistryId(value: string) {
    this.set("revocationRegistryId", Value.fromString(value));
  }

  get statusList_revocationList(): string {
    let value = this.get("statusList_revocationList");
    if (!value || value.kind == ValueKind.NULL) {
      throw new Error("Cannot return null for a required field.");
    } else {
      return value.toString();
    }
  }

  set statusList_revocationList(value: string) {
    this.set("statusList_revocationList", Value.fromString(value));
  }

  get statusList_currentAccumulator(): string {
    let value = this.get("statusList_currentAccumulator");
    if (!value || value.kind == ValueKind.NULL) {
      throw new Error("Cannot return null for a required field.");
    } else {
      return value.toString();
    }
  }

  set statusList_currentAccumulator(value: string) {
    this.set("statusList_currentAccumulator", Value.fromString(value));
  }

  get statusList_metadata_blockTimestamp(): BigInt {
    let value = this.get("statusList_metadata_blockTimestamp");
    if (!value || value.kind == ValueKind.NULL) {
      throw new Error("Cannot return null for a required field.");
    } else {
      return value.toBigInt();
    }
  }

  set statusList_metadata_blockTimestamp(value: BigInt) {
    this.set("statusList_metadata_blockTimestamp", Value.fromBigInt(value));
  }

  get statusList_metadata_blockNumber(): BigInt {
    let value = this.get("statusList_metadata_blockNumber");
    if (!value || value.kind == ValueKind.NULL) {
      throw new Error("Cannot return null for a required field.");
    } else {
      return value.toBigInt();
    }
  }

  set statusList_metadata_blockNumber(value: BigInt) {
    this.set("statusList_metadata_blockNumber", Value.fromBigInt(value));
  }

  get statusList_previousMetadata_blockTimestamp(): BigInt {
    let value = this.get("statusList_previousMetadata_blockTimestamp");
    if (!value || value.kind == ValueKind.NULL) {
      throw new Error("Cannot return null for a required field.");
    } else {
      return value.toBigInt();
    }
  }

  set statusList_previousMetadata_blockTimestamp(value: BigInt) {
    this.set(
      "statusList_previousMetadata_blockTimestamp",
      Value.fromBigInt(value)
    );
  }

  get statusList_previousMetadata_blockNumber(): BigInt {
    let value = this.get("statusList_previousMetadata_blockNumber");
    if (!value || value.kind == ValueKind.NULL) {
      throw new Error("Cannot return null for a required field.");
    } else {
      return value.toBigInt();
    }
  }

  set statusList_previousMetadata_blockNumber(value: BigInt) {
    this.set(
      "statusList_previousMetadata_blockNumber",
      Value.fromBigInt(value)
    );
  }

  get blockNumber(): BigInt {
    let value = this.get("blockNumber");
    if (!value || value.kind == ValueKind.NULL) {
      throw new Error("Cannot return null for a required field.");
    } else {
      return value.toBigInt();
    }
  }

  set blockNumber(value: BigInt) {
    this.set("blockNumber", Value.fromBigInt(value));
  }

  get blockTimestamp(): BigInt {
    let value = this.get("blockTimestamp");
    if (!value || value.kind == ValueKind.NULL) {
      throw new Error("Cannot return null for a required field.");
    } else {
      return value.toBigInt();
    }
  }

  set blockTimestamp(value: BigInt) {
    this.set("blockTimestamp", Value.fromBigInt(value));
  }

  get transactionHash(): Bytes {
    let value = this.get("transactionHash");
    if (!value || value.kind == ValueKind.NULL) {
      throw new Error("Cannot return null for a required field.");
    } else {
      return value.toBytes();
    }
  }

  set transactionHash(value: Bytes) {
    this.set("transactionHash", Value.fromBytes(value));
  }
}