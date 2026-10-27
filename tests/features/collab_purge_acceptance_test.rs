//! Acceptance test for Task 093: Remove Realtime Collaboration and Server Relay.
//! Verifies that the realtime collaboration feature and relay server integrations
//! are completely decommissioned and purged from the Kant desktop application.

use kant::ui::app::{App, MenuType};

#[test]
fn test_task_093_collab_and_relay_purged() {
    // AC4: Header menu må kun indeholde File, Export og Help (ingen Collab)
    assert_eq!(
        App::available_menu_types(),
        &[MenuType::File, MenuType::Export, MenuType::Help],
        "Header menus must only consist of File, Export, Help"
    );

    // AC1-AC4: Collab feature flag skal være deaktiveret/fjernet
    assert!(
        !App::has_collab_feature(),
        "Collab feature must be completely decommissioned and removed"
    );
}
