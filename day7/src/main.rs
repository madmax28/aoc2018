use std::collections::{HashMap, HashSet};
use std::fs;

type Task = char;

type Taskset = HashMap<Task, HashSet<Task>>;

#[derive(Debug)]
enum SchedulerError {
    NoRunnable,
}

#[derive(Debug)]
struct Scheduler {
    tasks: Taskset,
}

impl Scheduler {
    fn new(tasks: Taskset) -> Self {
        Scheduler { tasks }
    }

    fn schedule(&mut self) -> Result<Option<char>, SchedulerError> {
        if self.tasks.is_empty() {
            return Ok(None);
        }

        let mut runnables: Vec<char> = self
            .tasks
            .iter()
            .filter_map(
                |(task, deps)| {
                    if deps.is_empty() {
                        Some(task)
                    } else {
                        None
                    }
                },
            )
            .cloned()
            .collect();
        runnables.sort();

        let r = *runnables.get(0).ok_or(SchedulerError::NoRunnable)?;
        self.tasks.remove(&r).unwrap();

        Ok(Some(r))
    }

    fn schedule_workers(&mut self, workers: &mut [Worker]) {
        for w in workers {
            if w.is_busy() {
                continue;
            }

            if let Ok(Some(task)) = self.schedule() {
                w.assign(task);
            } else {
                break;
            }
        }
    }

    fn finish(&mut self, task: char) {
        for deps in self.tasks.values_mut() {
            deps.remove(&task);
        }
    }
}

#[derive(Debug, Clone)]
struct Worker {
    task: Option<Task>,
    time_left: u32,
}

impl Worker {
    fn new() -> Self {
        Worker {
            task: None,
            time_left: 0,
        }
    }

    fn assign(&mut self, task: Task) {
        // Sonst wird gestreikt!
        assert!(!self.is_busy());
        self.task = Some(task);
        self.time_left = 60 + task as u32 - 'A' as u32 + 1;
    }

    fn work(&mut self) -> Option<char> {
        assert!(self.time_left > 0);
        assert!(self.is_busy());
        if self.time_left > 0 {
            self.time_left -= 1;
        }

        if self.time_left == 0 {
            self.task.take()
        } else {
            None
        }
    }

    fn is_busy(&self) -> bool {
        self.task.is_some()
    }
}

fn main() -> Result<(), Box<std::error::Error>> {
    let input = fs::read_to_string("input")?;

    let mut tasks = Taskset::new();
    for line in input.lines() {
        let chars: Vec<char> = line
            .replace("Step ", "")
            .replace(" must be finished before step ", "")
            .replace(" can begin.", "")
            .chars()
            .collect();
        let dep = *chars.get(0).expect("error parsing line");
        let task = *chars.get(1).expect("error parsing line");
        tasks.entry(task).or_insert_with(HashSet::new).insert(dep);
        tasks.entry(dep).or_insert_with(HashSet::new);
    }

    let mut sched = Scheduler::new(tasks.clone());
    let mut order = String::new();
    while let Some(task) = sched.schedule().expect("deadlock?") {
        order.push(task);
        sched.finish(task);
    }
    println!("Part 1 order: {}", order);

    let mut sched = Scheduler::new(tasks);
    let mut workers = vec![Worker::new(); 5];
    let mut t = 0;
    sched.schedule_workers(&mut workers);
    while workers.iter().any(|w| w.is_busy()) {
        for w in &mut workers {
            if w.is_busy() {
                if let Some(task) = w.work() {
                    sched.finish(task);
                }
            }
        }
        sched.schedule_workers(&mut workers);
        t += 1;
    }
    println!("Part 2 time elapsed: {}", t);

    Ok(())
}
