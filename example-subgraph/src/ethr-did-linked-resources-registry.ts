import {
  ImmutableResourceCreatedEvent as ImmutableResourceCreatedEventEvent,
  MutableResourceUpdatedEvent as MutableResourceUpdatedEventEvent
} from "../generated/EthrDIDLinkedResourcesRegistry/EthrDIDLinkedResourcesRegistry"
import { ImmutableResourceCreatedEvent, MutableResourceUpdatedEvent } from "../generated/schema"

export function handleImmutableResourceCreatedEvent(event: ImmutableResourceCreatedEventEvent): void {
  let entity = new ImmutableResourceCreatedEvent(
    event.transaction.hash.concatI32(event.logIndex.toI32())
  )
  entity.didIdentity = event.params.didIdentity
  entity.name = event.params.name

  entity.blockNumber = event.block.number
  entity.blockTimestamp = event.block.timestamp
  entity.transactionHash = event.transaction.hash

  entity.save()
}

export function handleMutableResourceUpdatedEvent(
  event: MutableResourceUpdatedEventEvent
): void {
  let entity = new MutableResourceUpdatedEvent(
    event.transaction.hash.concatI32(event.logIndex.toI32())
  )

  entity.name = event.params.name
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
