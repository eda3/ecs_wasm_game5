// src/ecs/world_tests.rs
// World のユニットテスト！

// 親モジュール (World の定義がある場所) のアイテムを全部インポート！
use super::*;
// テストで使う標準ライブラリもインポート！
use std::any::TypeId;
use std::collections::{HashMap, HashSet}; // HashMap と HashSet を使う
use wasm_bindgen_test::*; // ★ wasm-bindgen-test をインポート ★★★
use crate::ecs::component::Component; // Component トレイトも使う
// ★ StackType も使う可能性があるのでインポート (find_entity_by_stack_type のテストなど) ★
use crate::components::stack::StackType; // ★追加★
// ★ StackInfo と Card もインポート ★
use crate::components::stack::StackInfo;
use crate::components::card::{Card, Suit, Rank};

// --- テスト用のダミーコンポーネントを定義 ---

// 位置情報を表すシンプルなコンポーネント
#[derive(Debug, Clone, Copy, PartialEq, Eq)] // テストで比較したり表示したりコピーしたりするので必要なトレイトを derive！
struct Position {
    x: i32,
    y: i32,
}
// Position がコンポーネントであることを示すマーカー実装！
impl Component for Position {}

// 速度情報を表すシンプルなコンポーネント
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Velocity {
    dx: i32,
    dy: i32,
}
// Velocity がコンポーネントであることを示すマーカー実装！
impl Component for Velocity {}

// --- テスト関数たち ---
// 各テスト関数には #[wasm_bindgen_test] アトリビュートを付けるよ！

#[wasm_bindgen_test]
fn test_new_world_is_empty() {
    let world = World::new();
    assert!(world.entities.is_empty(), "New world should have no entities");
    assert_eq!(world.next_entity_id, 0, "Next entity ID should start at 0");
    assert!(world.component_stores.is_empty(), "New world should have no component stores");
    // assert!(world.free_list.is_empty(), "New world should have an empty free list"); // free_list を使う場合はこれも
    println!("test_new_world_is_empty: PASSED ✅");
}

#[wasm_bindgen_test]
fn test_create_entity() {
    let mut world = World::new();
    let entity1 = world.create_entity();
    let entity2 = world.create_entity();

    assert_eq!(entity1, Entity(0), "First entity ID should be 0");
    assert_eq!(entity2, Entity(1), "Second entity ID should be 1");
    assert_eq!(world.next_entity_id, 2, "Next entity ID should be 2");
    assert_eq!(world.entities.len(), 2, "World should contain 2 entities");
    assert!(world.entities.contains(&entity1), "World should contain entity1");
    assert!(world.entities.contains(&entity2), "World should contain entity2");
    println!("test_create_entity: PASSED ✅");
}

#[wasm_bindgen_test]
fn test_create_entity_with_id() {
    let mut world = World::new();
    let entity5 = Entity(5);
    let entity2 = Entity(2);

    world.create_entity_with_id(entity5);
    assert!(world.is_entity_alive(entity5), "Entity 5 should be alive");
    assert_eq!(world.next_entity_id, 6, "Next ID should be 6 after adding entity 5");
    assert_eq!(world.entities.len(), 1, "World should have 1 entity");

    world.create_entity_with_id(entity2);
    assert!(world.is_entity_alive(entity2), "Entity 2 should be alive");
    assert_eq!(world.next_entity_id, 6, "Next ID should still be 6 after adding entity 2");
    assert_eq!(world.entities.len(), 2, "World should have 2 entities");

    // 通常の create_entity を呼ぶと、next_entity_id から新しい ID が使われる
    let entity6 = world.create_entity();
    assert_eq!(entity6, Entity(6), "Next created entity should have ID 6");
    assert_eq!(world.next_entity_id, 7, "Next ID should become 7");
    assert_eq!(world.entities.len(), 3, "World should have 3 entities");

    println!("test_create_entity_with_id: PASSED ✅");
}

#[wasm_bindgen_test]
fn test_is_entity_alive() {
    let mut world = World::new();
    let entity0 = world.create_entity();
    let entity1 = Entity(1); // まだ作ってない

    assert!(world.is_entity_alive(entity0), "Entity 0 should be alive");
    assert!(!world.is_entity_alive(entity1), "Entity 1 should not be alive yet");

    world.create_entity_with_id(entity1);
    assert!(world.is_entity_alive(entity1), "Entity 1 should be alive now");

    println!("test_is_entity_alive: PASSED ✅");
}

#[wasm_bindgen_test]
fn test_register_and_add_component() {
    let mut world = World::new();
    world.register_component::<Position>(); // Position 型のコンポーネントを使えるように登録！

    let entity1 = world.create_entity();
    let pos1 = Position { x: 10, y: 20 };
    world.add_component(entity1, pos1); // entity1 に Position コンポーネントを追加！

    // ComponentStoreEntry と remover の存在を確認 (内部的なテスト)
    let type_id_pos = TypeId::of::<Position>();
    assert!(world.component_stores.contains_key(&type_id_pos), "Position store should exist");
    let entry = world.component_stores.get(&type_id_pos).unwrap();
    assert!(entry.storage.is::<HashMap<Entity, Position>>(), "Storage should be HashMap<Entity, Position>");
    // entry.remover のテストは難しいので、destroy_entity のテストで間接的に確認する

    // ストレージから直接値を確認 (テスト用の storage メソッドを使う)
    let storage_any = world.storage::<Position>().expect("Position storage should exist");
    let storage_map = storage_any.downcast_ref::<HashMap<Entity, Position>>().expect("Should downcast to HashMap<Entity, Position>");

    assert_eq!(storage_map.len(), 1, "Position storage should have 1 entry");
    assert_eq!(storage_map.get(&entity1), Some(&pos1), "Stored position should match");
    assert_eq!(storage_map.len(), 1, "Storage size should remain 1 BEFORE adding to non-existent");

    // get_component で取得できるか確認
    assert_eq!(world.get_component::<Position>(entity1), Some(&pos1));

    // 存在しないエンティティに追加しようとしても何も起こらないはず
    let non_existent_entity = Entity(99);
    world.add_component(non_existent_entity, Position { x: 0, y: 0 });
    assert_eq!(world.get_component::<Position>(non_existent_entity), None);

    println!("test_register_and_add_component: PASSED ✅");
}


// #[test]
// #[should_panic] // このテストはパニックすることを期待してたけど、wasm_bindgen_test では直接サポートされてない
// fn test_add_component_unregistered() {
//     let mut world = World::new();
//     let entity1 = world.create_entity();
//     // Position を register せずに add しようとするとパニックするはず！
//     world.add_component(entity1, Position { x: 0, y: 0 });
//     // ここに到達したらテスト失敗！
// }

#[wasm_bindgen_test]
fn test_get_component() {
    let mut world = World::new();
    world.register_component::<Position>();
    world.register_component::<Velocity>();

    let entity1 = world.create_entity();
    let entity2 = world.create_entity();

    let pos1 = Position { x: 1, y: 2 };
    let vel1 = Velocity { dx: 3, dy: 4 };
    let pos2 = Position { x: 5, y: 6 };

    world.add_component(entity1, pos1);
    world.add_component(entity1, vel1); // 同じエンティティに複数のコンポーネントを追加
    world.add_component(entity2, pos2);

    // 正しく取得できるか
    assert_eq!(world.get_component::<Position>(entity1), Some(&pos1));
    assert_eq!(world.get_component::<Velocity>(entity1), Some(&vel1));
    assert_eq!(world.get_component::<Position>(entity2), Some(&pos2));

    // 持っていないコンポーネントは None
    assert_eq!(world.get_component::<Velocity>(entity2), None);

    // 存在しないエンティティは None
    assert_eq!(world.get_component::<Position>(Entity(99)), None);

    // 登録されていないコンポーネント型は None (パニックしない！)
    #[derive(Debug, Clone, Copy, PartialEq, Eq)] struct UnregisteredComponent;
    impl Component for UnregisteredComponent {}
    assert_eq!(world.get_component::<UnregisteredComponent>(entity1), None);

    println!("test_get_component: PASSED ✅");
}

#[wasm_bindgen_test]
fn test_get_component_mut() {
    let mut world = World::new();
    world.register_component::<Position>();

    let entity1 = world.create_entity();
    let pos1 = Position { x: 1, y: 2 };
    world.add_component(entity1, pos1);

    // 可変参照を取得して値を変更
    { // スコープを作って可変参照の寿命を制限する (Rust警察👮‍♀️対策！)
        let pos_mut = world.get_component_mut::<Position>(entity1).expect("Should get mutable position");
        pos_mut.x += 10;
        pos_mut.y += 20;
    } // ここで pos_mut の可変借用が終わる

    // 変更が反映されているか確認
    let expected_pos = Position { x: 11, y: 22 };
    assert_eq!(world.get_component::<Position>(entity1), Some(&expected_pos));

    // 持っていない、存在しない、登録されていない場合は None
    assert_eq!(world.get_component_mut::<Position>(Entity(99)), None);
    assert_eq!(world.get_component_mut::<Velocity>(entity1), None); // Velocity は登録されてない
    #[derive(Debug, PartialEq)] // <- PartialEq を追加
    struct Unregistered; impl Component for Unregistered {}
    assert_eq!(world.get_component_mut::<Unregistered>(entity1), None);

    println!("test_get_component_mut: PASSED ✅");
}

#[wasm_bindgen_test]
fn test_remove_component() {
    let mut world = World::new();
    world.register_component::<Position>();
    world.register_component::<Velocity>();

    let entity1 = world.create_entity();
    let pos1 = Position { x: 1, y: 2 };
    let vel1 = Velocity { dx: 3, dy: 4 };

    world.add_component(entity1, pos1);
    world.add_component(entity1, vel1);

    // Position を削除
    let removed_pos = world.remove_component::<Position>(entity1);
    assert_eq!(removed_pos, Some(pos1), "Removed position should match");
    assert_eq!(world.get_component::<Position>(entity1), None, "Position should be gone");
    assert!(world.storage::<Position>().unwrap().downcast_ref::<HashMap<Entity, Position>>().unwrap().get(&entity1).is_none(), "Position should be gone from storage map");


    // Velocity はまだ残っているはず
    assert_eq!(world.get_component::<Velocity>(entity1), Some(&vel1));

    // 存在しないコンポーネントを削除しようとしても None が返る
    let removed_again = world.remove_component::<Position>(entity1);
    assert_eq!(removed_again, None, "Removing again should return None");

    // 存在しないエンティティから削除しようとしても None
    assert_eq!(world.remove_component::<Velocity>(Entity(99)), None);

    // 登録されていないコンポーネント型を削除しようとしても None (パニックしない)
    #[derive(Debug, PartialEq)] struct Unregistered; impl Component for Unregistered {}
    assert_eq!(world.remove_component::<Unregistered>(entity1), None);

    println!("test_remove_component: PASSED ✅");
}


#[wasm_bindgen_test]
fn test_get_all_entities_with_component() {
    let mut world = World::new();
    world.register_component::<Position>();
    world.register_component::<Velocity>();

    let entity1 = world.create_entity(); // Pos, Vel
    let entity2 = world.create_entity(); // Pos
    let entity3 = world.create_entity(); // Vel
    let _entity4 = world.create_entity(); // None

    world.add_component(entity1, Position { x: 0, y: 0 });
    world.add_component(entity1, Velocity { dx: 0, dy: 0 });
    world.add_component(entity2, Position { x: 1, y: 1 });
    world.add_component(entity3, Velocity { dx: 2, dy: 2 });

    // Position を持つエンティティを取得
    let mut pos_entities = world.get_all_entities_with_component::<Position>();
    pos_entities.sort(); // 順序を保証するためにソート
    assert_eq!(pos_entities, vec![entity1, entity2]);

    // Velocity を持つエンティティを取得
    let mut vel_entities = world.get_all_entities_with_component::<Velocity>();
    vel_entities.sort(); // ソート
    assert_eq!(vel_entities, vec![entity1, entity3]);

    // 登録されていないコンポーネントは空の Vec
    #[derive(Debug)] struct Unregistered; impl Component for Unregistered {}
    assert!(world.get_all_entities_with_component::<Unregistered>().is_empty());

    println!("test_get_all_entities_with_component: PASSED ✅");
}

#[wasm_bindgen_test]
fn test_destroy_entity_removes_components() {
    let mut world = World::new();
    world.register_component::<Position>();
    world.register_component::<Velocity>();

    let entity1 = world.create_entity();
    let entity2 = world.create_entity();

    world.add_component(entity1, Position { x: 1, y: 1 });
    world.add_component(entity1, Velocity { dx: 1, dy: 1 });
    world.add_component(entity2, Position { x: 2, y: 2 });

    // entity1 を削除
    let destroyed = world.destroy_entity(entity1);
    assert!(destroyed, "Entity 1 should be destroyed");
    assert!(!world.is_entity_alive(entity1), "Entity 1 should not be alive");
    assert_eq!(world.entities.len(), 1, "Only entity 2 should remain");

    // entity1 のコンポーネントが削除されているか確認
    assert_eq!(world.get_component::<Position>(entity1), None, "Position for entity 1 should be gone");
    assert_eq!(world.get_component::<Velocity>(entity1), None, "Velocity for entity 1 should be gone");

    // ストレージからも消えているか確認
    assert!(world.storage::<Position>().unwrap().downcast_ref::<HashMap<Entity, Position>>().unwrap().get(&entity1).is_none(), "Pos map");
    assert!(world.storage::<Velocity>().unwrap().downcast_ref::<HashMap<Entity, Velocity>>().unwrap().get(&entity1).is_none(), "Vel map");


    // entity2 は影響を受けていないか確認
    assert!(world.is_entity_alive(entity2), "Entity 2 should still be alive");
    assert!(world.get_component::<Position>(entity2).is_some(), "Entity 2 should still have Position");
    assert!(world.storage::<Position>().unwrap().downcast_ref::<HashMap<Entity, Position>>().unwrap().get(&entity2).is_some(), "Pos map for entity 2");

    // 存在しないエンティティを削除しようとしても false が返る
    let not_destroyed = world.destroy_entity(Entity(99));
    assert!(!not_destroyed, "Destroying non-existent entity should return false");

    println!("test_destroy_entity_removes_components: PASSED ✅");
}


#[wasm_bindgen_test]
fn test_find_entity_by_stack_type() {
    let mut world = World::new();
    world.register_component::<StackInfo>();
    world.register_component::<Card>(); // Cardもダミーで必要

    let e1 = world.create_entity();
    world.add_component(e1, StackInfo::new(StackType::Tableau(0), 0));
    world.add_component(e1, Card::new(Suit::Heart, Rank::Ace)); // ダミー

    let e2 = world.create_entity();
    world.add_component(e2, StackInfo::new(StackType::Tableau(1), 0));
    world.add_component(e2, Card::new(Suit::Spade, Rank::Two)); // ダミー

    let e3 = world.create_entity();
    world.add_component(e3, StackInfo::new(StackType::Foundation(0), 0));
    world.add_component(e3, Card::new(Suit::Club, Rank::King)); // ダミー

    // StackInfo の position_in_stack が 0 で、StackType が Tableau(0) のエンティティを探す
    assert_eq!(world.find_entity_by_stack_type(StackType::Tableau(0)), Some(e1));
    assert_eq!(world.find_entity_by_stack_type(StackType::Tableau(1)), Some(e2));
    assert_eq!(world.find_entity_by_stack_type(StackType::Foundation(0)), Some(e3));
    assert_eq!(world.find_entity_by_stack_type(StackType::Stock), None); // Stock はまだない
    assert_eq!(world.find_entity_by_stack_type(StackType::Waste), None); // Waste はまだない
    assert_eq!(world.find_entity_by_stack_type(StackType::Foundation(1)), None); // Foundation(1) はまだない

    println!("test_find_entity_by_stack_type: PASSED ✅");
}


// TODO: free_list を実装したら、destroy -> create で ID が再利用されるかのテストも追加！
// #[wasm_bindgen_test]
// fn test_entity_id_reuse() { ... } 