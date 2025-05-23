use std::collections::VecDeque;

use super::{Object, ValidationError};

pub fn graph_valid(objects: &Vec<Object>) -> Result<(), ValidationError> {
    let mut graph: VecDeque<Vec<String>> = VecDeque::new();
    for obj in objects.iter() {
        graph.push_back(vec![obj.name.clone()]);
    }
    while let Some(eval) = graph.pop_front() {
        let eval_object: &Object = objects
            .iter()
            .find(|obj| *obj.name == *eval.last().unwrap())
            .unwrap();
        if eval_object
            .depends_on()
            .iter()
            .any(|dep| eval.contains(dep))
        {
            return Err(ValidationError::CircularDependancy(
                eval_object.name.clone(),
                eval.first().unwrap().clone(),
            ));
        } else {
            for dep in eval_object.depends_on() {
                let mut new_path = eval.clone();
                new_path.push(dep.clone());
                graph.push_back(new_path);
            }
        }
    }
    Ok(())
}
