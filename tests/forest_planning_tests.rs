//! Comprehensive tests for the Forest of Agents planning system

use helios_engine::{Agent, Config, ForestBuilder, LLMConfig, TaskItem, TaskPlan, TaskStatus};

/// Helper function to create a test config
fn create_test_config() -> Config {
    Config {
        llm: LLMConfig {
            model_name: "gpt-3.5-turbo".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: "test-key".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
        },
        #[cfg(feature = "local")]
        local: None,
    }
}

#[tokio::test]
async fn test_task_plan_creation() {
    let plan = TaskPlan::new("plan_1".to_string(), "Test objective".to_string());

    assert_eq!(plan.plan_id, "plan_1");
    assert_eq!(plan.objective, "Test objective");
    assert_eq!(plan.tasks.len(), 0);
    assert_eq!(plan.task_order.len(), 0);
}

#[tokio::test]
async fn test_task_plan_add_task() {
    let mut plan = TaskPlan::new("plan_1".to_string(), "Test objective".to_string());

    let task1 = TaskItem::new(
        "task_1".to_string(),
        "First task".to_string(),
        "agent1".to_string(),
    );

    let task2 = TaskItem::new(
        "task_2".to_string(),
        "Second task".to_string(),
        "agent2".to_string(),
    );

    plan.add_task(task1);
    plan.add_task(task2);

    assert_eq!(plan.tasks.len(), 2);
    assert_eq!(plan.task_order.len(), 2);
    assert_eq!(plan.task_order[0], "task_1");
    assert_eq!(plan.task_order[1], "task_2");
}

#[tokio::test]
async fn test_task_plan_get_task() {
    let mut plan = TaskPlan::new("plan_1".to_string(), "Test objective".to_string());

    let task = TaskItem::new(
        "task_1".to_string(),
        "Test task".to_string(),
        "agent1".to_string(),
    );

    plan.add_task(task);

    // Test get_task
    let retrieved = plan.get_task("task_1");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, "task_1");

    // Test non-existent task
    let missing = plan.get_task("task_99");
    assert!(missing.is_none());
}

#[tokio::test]
async fn test_task_plan_get_task_mut() {
    let mut plan = TaskPlan::new("plan_1".to_string(), "Test objective".to_string());

    let task = TaskItem::new(
        "task_1".to_string(),
        "Test task".to_string(),
        "agent1".to_string(),
    );

    plan.add_task(task);

    // Modify task via mutable reference
    if let Some(task_mut) = plan.get_task_mut("task_1") {
        task_mut.status = TaskStatus::Completed;
        task_mut.result = Some("Task completed".to_string());
    }

    // Verify changes
    let task = plan.get_task("task_1").unwrap();
    assert_eq!(task.status, TaskStatus::Completed);
    assert_eq!(task.result.as_ref().unwrap(), "Task completed");
}

#[tokio::test]
async fn test_task_dependencies() {
    let mut plan = TaskPlan::new("plan_1".to_string(), "Test objective".to_string());

    let task1 = TaskItem::new(
        "task_1".to_string(),
        "First task".to_string(),
        "agent1".to_string(),
    );

    let task2 = TaskItem::new(
        "task_2".to_string(),
        "Second task".to_string(),
        "agent2".to_string(),
    )
    .with_dependencies(vec!["task_1".to_string()]);

    plan.add_task(task1);
    plan.add_task(task2);

    // Verify dependencies
    let task2_retrieved = plan.get_task("task_2").unwrap();
    assert_eq!(task2_retrieved.dependencies.len(), 1);
    assert_eq!(task2_retrieved.dependencies[0], "task_1");
}

#[tokio::test]
async fn test_get_next_ready_tasks_no_dependencies() {
    let mut plan = TaskPlan::new("plan_1".to_string(), "Test objective".to_string());

    let task1 = TaskItem::new(
        "task_1".to_string(),
        "First task".to_string(),
        "agent1".to_string(),
    );

    let task2 = TaskItem::new(
        "task_2".to_string(),
        "Second task".to_string(),
        "agent2".to_string(),
    );

    plan.add_task(task1);
    plan.add_task(task2);

    // Both tasks should be ready (no dependencies)
    let ready = plan.get_next_ready_tasks();
    assert_eq!(ready.len(), 2);
}

#[tokio::test]
async fn test_get_next_ready_tasks_with_dependencies() {
    let mut plan = TaskPlan::new("plan_1".to_string(), "Test objective".to_string());

    let task1 = TaskItem::new(
        "task_1".to_string(),
        "First task".to_string(),
        "agent1".to_string(),
    );

    let task2 = TaskItem::new(
        "task_2".to_string(),
        "Second task".to_string(),
        "agent2".to_string(),
    )
    .with_dependencies(vec!["task_1".to_string()]);

    plan.add_task(task1);
    plan.add_task(task2);

    // Only task1 should be ready
    let ready = plan.get_next_ready_tasks();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, "task_1");

    // Complete task1
    plan.get_task_mut("task_1").unwrap().status = TaskStatus::Completed;

    // Now task2 should be ready
    let ready = plan.get_next_ready_tasks();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, "task_2");
}

#[tokio::test]
async fn test_get_next_ready_tasks_complex_dependencies() {
    let mut plan = TaskPlan::new("plan_1".to_string(), "Test objective".to_string());

    // Create a dependency chain: task1 -> task2 -> task3
    //                             task1 -> task4
    let task1 = TaskItem::new(
        "task_1".to_string(),
        "Task 1".to_string(),
        "agent1".to_string(),
    );
    let task2 = TaskItem::new(
        "task_2".to_string(),
        "Task 2".to_string(),
        "agent2".to_string(),
    )
    .with_dependencies(vec!["task_1".to_string()]);
    let task3 = TaskItem::new(
        "task_3".to_string(),
        "Task 3".to_string(),
        "agent3".to_string(),
    )
    .with_dependencies(vec!["task_2".to_string()]);
    let task4 = TaskItem::new(
        "task_4".to_string(),
        "Task 4".to_string(),
        "agent4".to_string(),
    )
    .with_dependencies(vec!["task_1".to_string()]);

    plan.add_task(task1);
    plan.add_task(task2);
    plan.add_task(task3);
    plan.add_task(task4);

    // Initially only task1 ready
    let ready = plan.get_next_ready_tasks();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, "task_1");

    // Complete task1
    plan.get_task_mut("task_1").unwrap().status = TaskStatus::Completed;

    // Now task2 and task4 should be ready
    let ready = plan.get_next_ready_tasks();
    assert_eq!(ready.len(), 2);
    let ready_ids: Vec<_> = ready.iter().map(|t| &t.id).collect();
    assert!(ready_ids.contains(&&"task_2".to_string()));
    assert!(ready_ids.contains(&&"task_4".to_string()));

    // Complete task2
    plan.get_task_mut("task_2").unwrap().status = TaskStatus::Completed;

    // Now task3 should be ready (task4 still ready too)
    let ready = plan.get_next_ready_tasks();
    assert_eq!(ready.len(), 2);
}

#[tokio::test]
async fn test_task_plan_is_complete() {
    let mut plan = TaskPlan::new("plan_1".to_string(), "Test objective".to_string());

    let task1 = TaskItem::new(
        "task_1".to_string(),
        "Task 1".to_string(),
        "agent1".to_string(),
    );
    let task2 = TaskItem::new(
        "task_2".to_string(),
        "Task 2".to_string(),
        "agent2".to_string(),
    );

    plan.add_task(task1);
    plan.add_task(task2);

    // Not complete initially
    assert!(!plan.is_complete());

    // Complete one task
    plan.get_task_mut("task_1").unwrap().status = TaskStatus::Completed;
    assert!(!plan.is_complete());

    // Complete second task
    plan.get_task_mut("task_2").unwrap().status = TaskStatus::Completed;
    assert!(plan.is_complete());
}

#[tokio::test]
async fn test_task_plan_get_progress() {
    let mut plan = TaskPlan::new("plan_1".to_string(), "Test objective".to_string());

    let task1 = TaskItem::new(
        "task_1".to_string(),
        "Task 1".to_string(),
        "agent1".to_string(),
    );
    let task2 = TaskItem::new(
        "task_2".to_string(),
        "Task 2".to_string(),
        "agent2".to_string(),
    );
    let task3 = TaskItem::new(
        "task_3".to_string(),
        "Task 3".to_string(),
        "agent3".to_string(),
    );

    plan.add_task(task1);
    plan.add_task(task2);
    plan.add_task(task3);

    // 0/3 completed
    let (completed, total) = plan.get_progress();
    assert_eq!(completed, 0);
    assert_eq!(total, 3);

    // Complete one task
    plan.get_task_mut("task_1").unwrap().status = TaskStatus::Completed;
    let (completed, total) = plan.get_progress();
    assert_eq!(completed, 1);
    assert_eq!(total, 3);

    // Complete two tasks
    plan.get_task_mut("task_2").unwrap().status = TaskStatus::Completed;
    let (completed, total) = plan.get_progress();
    assert_eq!(completed, 2);
    assert_eq!(total, 3);
}

#[tokio::test]
async fn test_tasks_in_order() {
    let mut plan = TaskPlan::new("plan_1".to_string(), "Test objective".to_string());

    let task1 = TaskItem::new(
        "task_1".to_string(),
        "First".to_string(),
        "agent1".to_string(),
    );
    let task2 = TaskItem::new(
        "task_2".to_string(),
        "Second".to_string(),
        "agent2".to_string(),
    );
    let task3 = TaskItem::new(
        "task_3".to_string(),
        "Third".to_string(),
        "agent3".to_string(),
    );

    plan.add_task(task1);
    plan.add_task(task2);
    plan.add_task(task3);

    let tasks = plan.tasks_in_order();
    assert_eq!(tasks.len(), 3);
    assert_eq!(tasks[0].id, "task_1");
    assert_eq!(tasks[1].id, "task_2");
    assert_eq!(tasks[2].id, "task_3");
}

#[tokio::test]
async fn test_task_status_as_str() {
    assert_eq!(TaskStatus::Pending.as_str(), "pending");
    assert_eq!(TaskStatus::InProgress.as_str(), "in_progress");
    assert_eq!(TaskStatus::Completed.as_str(), "completed");
    assert_eq!(TaskStatus::Failed.as_str(), "failed");
}

#[tokio::test]
async fn test_shared_context_plan_management() {
    use helios_engine::SharedContext;

    let mut context = SharedContext::new();

    // Initially no plan
    assert!(context.get_plan().is_none());

    // Set a plan
    let plan = TaskPlan::new("plan_1".to_string(), "Test objective".to_string());
    context.set_plan(plan.clone());

    assert!(context.get_plan().is_some());
    assert_eq!(context.get_plan().unwrap().plan_id, "plan_1");

    // Modify plan
    if let Some(plan_mut) = context.get_plan_mut() {
        let task = TaskItem::new(
            "task_1".to_string(),
            "Task".to_string(),
            "agent1".to_string(),
        );
        plan_mut.add_task(task);
    }

    assert_eq!(context.get_plan().unwrap().tasks.len(), 1);

    // Clear plan
    context.clear_plan();
    assert!(context.get_plan().is_none());
}

#[tokio::test]
async fn test_forest_with_planning_tools() {
    let config = create_test_config();

    let forest = ForestBuilder::new()
        .config(config)
        .agent(
            "coordinator".to_string(),
            Agent::builder("coordinator").system_prompt("Test coordinator"),
        )
        .agent(
            "worker".to_string(),
            Agent::builder("worker").system_prompt("Test worker"),
        )
        .build()
        .await
        .expect("Failed to build forest");

    // Verify agents were created
    let context = forest.get_shared_context().await;
    assert!(context.get_plan().is_none());
}

#[tokio::test]
async fn test_task_item_with_metadata() {
    let mut task = TaskItem::new(
        "task_1".to_string(),
        "Test task".to_string(),
        "agent1".to_string(),
    );

    task.metadata
        .insert("priority".to_string(), "high".to_string());
    task.metadata
        .insert("category".to_string(), "research".to_string());

    assert_eq!(task.metadata.len(), 2);
    assert_eq!(task.metadata.get("priority").unwrap(), "high");
    assert_eq!(task.metadata.get("category").unwrap(), "research");
}

#[tokio::test]
async fn test_performance_with_many_tasks() {
    // Test that O(T * D) performance is reasonable
    let mut plan = TaskPlan::new("plan_1".to_string(), "Performance test".to_string());

    // Create 100 tasks with dependencies
    for i in 0..100 {
        let deps = if i > 0 {
            vec![format!("task_{}", i - 1)]
        } else {
            vec![]
        };

        let task = TaskItem::new(
            format!("task_{}", i),
            format!("Task {}", i),
            "agent".to_string(),
        )
        .with_dependencies(deps);

        plan.add_task(task);
    }

    // This should be fast with HashMap implementation
    let start = std::time::Instant::now();
    let ready = plan.get_next_ready_tasks();
    let duration = start.elapsed();

    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, "task_0");

    // Should complete in well under 1ms with O(T * D) vs much longer with O(T²)
    assert!(
        duration.as_millis() < 10,
        "Performance issue detected: took {:?}",
        duration
    );
}

#[tokio::test]
async fn test_multiple_independent_tasks() {
    let mut plan = TaskPlan::new("plan_1".to_string(), "Test parallel tasks".to_string());

    // Create 5 independent tasks (no dependencies)
    for i in 0..5 {
        let task = TaskItem::new(
            format!("task_{}", i),
            format!("Task {}", i),
            format!("agent_{}", i),
        );
        plan.add_task(task);
    }

    // All should be ready
    let ready = plan.get_next_ready_tasks();
    assert_eq!(ready.len(), 5);
}

#[tokio::test]
async fn test_failed_task_completes_plan() {
    let mut plan = TaskPlan::new("plan_1".to_string(), "Test with failure".to_string());

    let task1 = TaskItem::new(
        "task_1".to_string(),
        "Task 1".to_string(),
        "agent1".to_string(),
    );
    let task2 = TaskItem::new(
        "task_2".to_string(),
        "Task 2".to_string(),
        "agent2".to_string(),
    );

    plan.add_task(task1);
    plan.add_task(task2);

    // Complete one, fail the other
    plan.get_task_mut("task_1").unwrap().status = TaskStatus::Completed;
    plan.get_task_mut("task_2").unwrap().status = TaskStatus::Failed;

    // Plan should be considered complete (all tasks in terminal state)
    assert!(plan.is_complete());
}

#[tokio::test]
async fn test_in_progress_tasks_not_ready() {
    let mut plan = TaskPlan::new("plan_1".to_string(), "Test in progress".to_string());

    let task1 = TaskItem::new(
        "task_1".to_string(),
        "Task 1".to_string(),
        "agent1".to_string(),
    );

    plan.add_task(task1);

    // Set to in progress
    plan.get_task_mut("task_1").unwrap().status = TaskStatus::InProgress;

    // Should not be in ready list
    let ready = plan.get_next_ready_tasks();
    assert_eq!(ready.len(), 0);
}

#[tokio::test]
async fn test_hashmap_provides_o1_lookup() {
    // Test that HashMap provides O(1) lookup by task_id
    let mut plan = TaskPlan::new("plan_1".to_string(), "HashMap test".to_string());

    // Add 1000 tasks
    for i in 0..1000 {
        let task = TaskItem::new(
            format!("task_{}", i),
            format!("Task {}", i),
            "agent".to_string(),
        );
        plan.add_task(task);
    }

    // Lookup should be fast regardless of position
    let start = std::time::Instant::now();

    // Lookup first, middle, and last tasks
    assert!(plan.get_task("task_0").is_some());
    assert!(plan.get_task("task_500").is_some());
    assert!(plan.get_task("task_999").is_some());

    let duration = start.elapsed();

    // All lookups should complete in microseconds, not milliseconds
    assert!(
        duration.as_micros() < 100,
        "HashMap lookup too slow: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_task_order_preserved() {
    // Verify that task_order preserves insertion order
    let mut plan = TaskPlan::new("plan_1".to_string(), "Order test".to_string());

    // Add tasks in specific order
    for i in 0..10 {
        let task = TaskItem::new(
            format!("task_{}", i),
            format!("Task {}", i),
            "agent".to_string(),
        );
        plan.add_task(task);
    }

    // Verify order is preserved
    let tasks = plan.tasks_in_order();
    for (idx, task) in tasks.iter().enumerate() {
        assert_eq!(task.id, format!("task_{}", idx));
    }
}

#[tokio::test]
async fn test_complex_dependency_chain() {
    // Test a complex dependency scenario with multiple paths
    let mut plan = TaskPlan::new("plan_1".to_string(), "Complex dependencies".to_string());

    // Create a diamond dependency pattern:
    //     task1
    //    /     \
    // task2   task3
    //    \     /
    //     task4

    let task1 = TaskItem::new("task1".to_string(), "Root".to_string(), "agent".to_string());
    let task2 = TaskItem::new("task2".to_string(), "Left".to_string(), "agent".to_string())
        .with_dependencies(vec!["task1".to_string()]);
    let task3 = TaskItem::new(
        "task3".to_string(),
        "Right".to_string(),
        "agent".to_string(),
    )
    .with_dependencies(vec!["task1".to_string()]);
    let task4 = TaskItem::new(
        "task4".to_string(),
        "Bottom".to_string(),
        "agent".to_string(),
    )
    .with_dependencies(vec!["task2".to_string(), "task3".to_string()]);

    plan.add_task(task1);
    plan.add_task(task2);
    plan.add_task(task3);
    plan.add_task(task4);

    // Only task1 should be ready
    let ready = plan.get_next_ready_tasks();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, "task1");

    // Complete task1
    plan.get_task_mut("task1").unwrap().status = TaskStatus::Completed;

    // task2 and task3 should be ready
    let ready = plan.get_next_ready_tasks();
    assert_eq!(ready.len(), 2);

    // Complete task2 but not task3
    plan.get_task_mut("task2").unwrap().status = TaskStatus::Completed;

    // task3 still ready, task4 not ready yet
    let ready = plan.get_next_ready_tasks();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, "task3");

    // Complete task3
    plan.get_task_mut("task3").unwrap().status = TaskStatus::Completed;

    // Now task4 should be ready (both dependencies satisfied)
    let ready = plan.get_next_ready_tasks();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, "task4");
}
