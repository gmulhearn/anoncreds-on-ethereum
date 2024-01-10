import {
  NewResourceEvent as NewResourceEventEvent,
  MutableResourceUpdateEvent as MutableResourceUpdateEventEvent
} from "../generated/EthrDIDLinkedResourcesRegistry/EthrDIDLinkedResourcesRegistry"
import { NewResourceEvent, MutableResourceUpdateEvent } from "../generated/schema"

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

export function handleMutableResourceUpdateEvent(
  event: MutableResourceUpdateEventEvent
): void {
  let entity = new MutableResourceUpdateEvent(
    event.transaction.hash.concatI32(event.logIndex.toI32())
  )

  entity.path = event.params.path
  entity.didIdentity = event.params.didIdentity

  entity.resource_content = event.params.resource.content
  entity.resource_metadata_blockNumber = event.params.resource.metadata.blockNumber
  entity.resource_metadata_blockTimestamp = event.params.resource.metadata.blockTimestamp
  entity.resource_previousMetadata_blockNumber = event.params.resource.previousMetadata.blockNumber
  entity.resource_previousMetadata_blockTimestamp = event.params.resource.previousMetadata.blockTimestamp

  entity.blockNumber = event.block.number
  entity.blockTimestamp = event.block.timestamp
  entity.transactionHash = event.transaction.hash

  entity.save()
}
