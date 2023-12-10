import {
  assert,
  describe,
  test,
  clearStore,
  beforeAll,
  afterAll
} from "matchstick-as/assembly/index"
import { Address } from "@graphprotocol/graph-ts"
import { NewResourceEvent } from "../generated/schema"
import { NewResourceEvent as NewResourceEventEvent } from "../generated/AnoncredsRegistry/AnoncredsRegistry"
import { handleNewResourceEvent } from "../src/anoncreds-registry"
import { createNewResourceEventEvent } from "./anoncreds-registry-utils"

// Tests structure (matchstick-as >=0.5.0)
// https://thegraph.com/docs/en/developer/matchstick/#tests-structure-0-5-0

describe("Describe entity assertions", () => {
  beforeAll(() => {
    let didIdentity = Address.fromString(
      "0x0000000000000000000000000000000000000001"
    )
    let path = "Example string value"
    let newNewResourceEventEvent = createNewResourceEventEvent(
      didIdentity,
      path
    )
    handleNewResourceEvent(newNewResourceEventEvent)
  })

  afterAll(() => {
    clearStore()
  })

  // For more test scenarios, see:
  // https://thegraph.com/docs/en/developer/matchstick/#write-a-unit-test

  test("NewResourceEvent created and stored", () => {
    assert.entityCount("NewResourceEvent", 1)

    // 0xa16081f360e3847006db660bae1c6d1b2e17ec2a is the default address used in newMockEvent() function
    assert.fieldEquals(
      "NewResourceEvent",
      "0xa16081f360e3847006db660bae1c6d1b2e17ec2a-1",
      "didIdentity",
      "0x0000000000000000000000000000000000000001"
    )
    assert.fieldEquals(
      "NewResourceEvent",
      "0xa16081f360e3847006db660bae1c6d1b2e17ec2a-1",
      "path",
      "Example string value"
    )

    // More assert options:
    // https://thegraph.com/docs/en/developer/matchstick/#asserts
  })
})
