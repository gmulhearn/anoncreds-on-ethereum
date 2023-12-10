import {
  NewResourceEvent as NewResourceEventEvent,
  StatusListUpdateEvent as StatusListUpdateEventEvent
} from "../generated/AnoncredsRegistry/AnoncredsRegistry"
import { NewResourceEvent, StatusListUpdateEvent } from "../generated/schema"

export function handleNewResourceEvent(event: NewResourceEventEvent): void {
  let entity = new NewResourceEvent(
    event.transaction.hash.concatI32(event.logIndex.toI32())
  )
  entity.didIdentity = event.params.didIdentity
  entity.path = event.params.path

  entity.blockNumber = event.block.number
  entity.blockTimestamp = event.block.timestamp
  entity.transactionHash = event.transaction.hash

  entity.save()
}

export function handleStatusListUpdateEvent(
  event: StatusListUpdateEventEvent
): void {
  let entity = new StatusListUpdateEvent(
    event.transaction.hash.concatI32(event.logIndex.toI32())
  )
  entity.indexedRevocationRegistryId = event.params.indexedRevocationRegistryId
  entity.revocationRegistryId = event.params.revocationRegistryId
  entity.statusList_revocationList = event.params.statusList.revocationList
  entity.statusList_currentAccumulator =
    event.params.statusList.currentAccumulator
  entity.statusList_metadata_blockTimestamp =
    event.params.statusList.metadata.blockTimestamp
  entity.statusList_metadata_blockNumber =
    event.params.statusList.metadata.blockNumber
  entity.statusList_previousMetadata_blockTimestamp =
    event.params.statusList.previousMetadata.blockTimestamp
  entity.statusList_previousMetadata_blockNumber =
    event.params.statusList.previousMetadata.blockNumber

  entity.blockNumber = event.block.number
  entity.blockTimestamp = event.block.timestamp
  entity.transactionHash = event.transaction.hash

  entity.save()
}
