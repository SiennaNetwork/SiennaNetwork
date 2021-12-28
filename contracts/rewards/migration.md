# Migration flow

Consider two contracts, **NEXT** and **PREV**

## Preparation

- Admin of **NEXT** calls **NEXT::EnableMigrationFrom(PREV)**.
  **NEXT/CAN_MIGRATE_FROM/PREV** is set to **Some(PREV)**.
- Admin of **PREV** calls **PREV::EnableMigrationTo(NEXT)**.
  **PREV/CAN_MIGRATE_TO/NEXT** is set to **Some(NEXT)**.

## Execution

- **USER** calls **NEXT::RequestMigration(PREV)**.
  - **NEXT::handle** calls **NEXT::can_immigrate_from(PREV)**.
    - If **NEXT/CAN_MIGRATE_FROM/PREV** is not **Some(_)**,
      the transaction fails here.
- **NEXT** calls **PREV::ExportState(USER)**.
  - **PREV::handle** calls **PREV::can_export_state(USER)**
    - The value of **env.message.sender** is equal to **the address of **NEXT**.**
    - If **PREV/CAN_MIGRATE_TO/NEXT** is not **Some(_)**,
      the transaction fails here.
- **PREV** calls **NEXT::ReceiveMigration(USER, SNAPSHOT)**.
  - **NEXT::handle** calls **NEXT::can_immigrate_from(PREV)** again.
    - This was what was missing in **HAL-01**.
    - If **NEXT/CAN_MIGRATE_FROM/PREV** is not **Some(_)**,
      the transaction fails here.
