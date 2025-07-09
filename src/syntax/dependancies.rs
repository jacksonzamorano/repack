use std::collections::VecDeque;

use super::{Object, RepackError, RepackErrorKind};

pub fn graph_valid(objects: &[Object]) -> Result<(), RepackError> {
    let mut graph: VecDeque<Vec<String>> = VecDeque::new();
    for obj in objects.iter() {
        graph.push_back(vec![obj.name.clone()]);
    }
    while let Some(eval) = graph.pop_front() {
        let Some(eval_object) = objects
            .iter()
            .find(|obj| *obj.name == *eval.last().unwrap())
        else {
            return Err(RepackError::global(
                RepackErrorKind::UnknownObject,
                format!("'{}' => '{}'", eval.last().unwrap(), eval.first().unwrap()),
            ));
        };
        if let Some(error) = eval_object
            .depends_on()
            .iter()
            .find(|dep| eval.contains(dep))
        {
            return Err(RepackError::from_obj_with_msg(
                RepackErrorKind::CircularDependancy,
                eval_object,
                error.to_string(),
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
