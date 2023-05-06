use std::cmp;
use crate::db::models::Task;

#[derive(Debug)]
struct TaskScored {
    id: String,
    name: String,
    description: String,
    status: i32,
    due: Option<chrono::NaiveDateTime>,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
    distance: usize,
}

fn min_of_three(a: usize, b: usize, c: usize) -> usize {
    return cmp::min(cmp::min(a, b), c)
} 


pub fn wagner_fisher(source: &str, target: &str) -> usize {
    let mut s_ = String::from(" ");
    let mut t_ = String::from(" ");
    s_.push_str(source);
    t_.push_str(target);
    let s = s_.as_bytes();
    let t = t_.as_bytes();
     
    
    //lengths
    let m = s.len();
    let n = t.len();

    //zero initialize 2d matrix    
    let mut d = vec![vec![0; n]; m];
    //source to target prefixes is always just a dropping of char
    for i in 1..m {
        d[i][0] = i;
    }
    //the reverse is true for target    
    for j in 1..n {
        d[0][j] = j;
    }
    
    let mut sub_cost;
    for j in 1..n {
        for i in 1..m {
            if s[i] == t[j] {
                sub_cost = 0;
            } else {
                sub_cost = 1;
            }
        d[i][j] = min_of_three(
            d[i-1][j] + 1,
            d[i][j-1] + 1, 
            d[i-1][j-1] + sub_cost as usize
            );
        }
    }
    return d[m -1][n-1]
}


fn get_min_distance_from_words(words: &str, term: &str) -> usize {
    let words_split = words.split(" ");
    let mut min_dist = -1;
    for w in words_split {
        let dist = wagner_fisher(w, term);
        if min_dist == -1 {
            min_dist = dist as isize;
        } else {
            min_dist = cmp::min(min_dist, dist as isize);
        }
    }
    min_dist as usize
}

impl Into<Task> for TaskScored {
    fn into(self) -> Task {
        Task { 
            id: self.id, 
            name: self.name, 
            description: self.description, 
            status: self.status, 
            due: self.due, 
            created_at: self.created_at, 
            updated_at: self.updated_at 
        }
    }
}


impl TaskScored {
    fn from_task_with_score(task: Task, term: &str) -> TaskScored {
        let dist_name = get_min_distance_from_words(&task.name, term);
        let dist_desc = get_min_distance_from_words(&task.description, term);
        let distance = cmp::min(dist_name, dist_desc);
        Self {
            id: task.id,
            name: task.name,
            description: task.description,
            status: task.status,
            due: task.due,
            created_at: task.created_at,
            updated_at: task.updated_at,
            distance,
        }
    }


}



pub fn sort_by_score(tasks: Vec<Task>, term: &str) -> Vec<Task> {
    let mut scored = tasks
        .into_iter()
        .map(|task| TaskScored::from_task_with_score(task, term))
        .collect::<Vec<TaskScored>>();

        scored.sort_by(|a, b| {
            a.distance.cmp(&b.distance)
        });
        scored.into_iter().map(|task| task.into()).collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_wagner_fisher() {
        let distance = wagner_fisher("Sunday", "Saturday");
        assert_eq!(distance, 3);
    }

    #[test] 
    fn test_sorting() {
        let task1 = Task::new("hello world", None, None);
        let tasks: Vec<Task> = vec![
            task1.clone(),
            Task::new("There is no target in the name", Some("we are helping everyone in hell"), None),
            Task::new("help is hello", None, None)
        ];
        let term = "help";
        let sorted = sort_by_score(tasks, term);
        let target_names = vec!["help is hello", "There is no target in the name", "hello world"];
        for i in 0..3 {
            assert_eq!(sorted[i].name, target_names[i])
        }
        assert_eq!(task1, sorted[2])
    }

}




