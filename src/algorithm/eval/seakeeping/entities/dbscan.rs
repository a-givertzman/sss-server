#[derive(Debug, Clone, PartialEq)]
pub enum PointType {
    Core,
    Border,
    Noise,
}

#[derive(Debug, Clone)]
pub struct DBSCANPoint {
    pub coords: (f64, f64),
    pub point_type: PointType,
    pub cluster_id: Option<usize>,
    pub visited: bool,
}

pub fn dbscan(points: &[(f64, f64)], eps: f64, min_points: usize) -> Vec<Vec<(f64, f64)>> {
    let mut dbscan_points: Vec<DBSCANPoint> = points
        .iter()
        .map(|&coords| DBSCANPoint {
            coords,
            point_type: PointType::Noise,
            cluster_id: None,
            visited: false,
        })
        .collect();
    
    let mut cluster_id = 0;
    
    for i in 0..dbscan_points.len() {
        if dbscan_points[i].visited {
            continue;
        }
        
        dbscan_points[i].visited = true;
        
        let neighbors = region_query(&dbscan_points, i, eps);
        
        if neighbors.len() < min_points {
            dbscan_points[i].point_type = PointType::Noise;
        } else {
            cluster_id += 1;
            expand_cluster(&mut dbscan_points, i, &neighbors, cluster_id, eps, min_points);
        }
    }
    let mut clusters: Vec<Vec<(f64, f64)>> = Vec::new();
    for _ in 0..=cluster_id {
        clusters.push(Vec::new());
    }
    for point in &dbscan_points {
        if let Some(id) = point.cluster_id
            && id > 0 {
                clusters[id - 1].push(point.coords);
            }
    }
    
    // Удаление пустых кластеров
    clusters.retain(|cluster| !cluster.is_empty());
    clusters
}

fn region_query(points: &[DBSCANPoint], point_idx: usize, eps: f64) -> Vec<usize> {
    let mut neighbors = Vec::new();
    let point_coords = points[point_idx].coords;
    
    for (i, other_point) in points.iter().enumerate() {
        if i != point_idx && distance(point_coords, other_point.coords) <= eps {
            neighbors.push(i);
        }
    }
    
    neighbors
}

fn expand_cluster(points: &mut [DBSCANPoint],
                 point_idx: usize,
                 neighbors: &[usize],
                 cluster_id: usize,
                 eps: f64,
                 min_points: usize) {
    points[point_idx].cluster_id = Some(cluster_id);
    points[point_idx].point_type = PointType::Core;
    
    let mut queue = neighbors.to_vec();
    
    while let Some(current_idx) = queue.pop() {
        if !points[current_idx].visited {
            points[current_idx].visited = true;
            let current_neighbors = region_query(points, current_idx, eps);
            
            if current_neighbors.len() >= min_points {
                points[current_idx].point_type = PointType::Core;
                for neighbor_idx in current_neighbors {
                    if points[neighbor_idx].cluster_id.is_none() {
                        queue.push(neighbor_idx);
                    }
                }
            } else {
                points[current_idx].point_type = PointType::Border;
            }
        }
        
        if points[current_idx].cluster_id.is_none() {
            points[current_idx].cluster_id = Some(cluster_id);
        }
    }
}

fn distance(a: (f64, f64), b: (f64, f64)) -> f64 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}