use nalgebra::{point, vector, Point2, Point3, Vector2, Vector3};
use thiserror::Error;
use types::CameraMatrix;

#[derive(Debug, Error)]
pub enum Error {
    #[error("position is too close to the camera to calculate")]
    TooClose,
    #[error("position is behind the camera")]
    BehindCamera,
    #[error("the pixel position is above the horion and cannot be projected to the ground")]
    AboveHorizon,
}

pub trait Projection {
    fn pixel_to_camera(&self, pixel_coordinates: Point2<f32>) -> Vector3<f32>;
    fn camera_to_pixel(&self, camera_ray: Vector3<f32>) -> Result<Point2<f32>, Error>;
    fn pixel_to_ground(&self, pixel_coordinates: Point2<f32>) -> Result<Point2<f32>, Error>;
    fn pixel_to_ground_with_z(
        &self,
        pixel_coordinates: Point2<f32>,
        z: f32,
    ) -> Result<Point2<f32>, Error>;
    fn ground_to_pixel(&self, ground_coordinates: Point2<f32>) -> Result<Point2<f32>, Error>;
    fn ground_with_z_to_pixel(
        &self,
        ground_coordinates: Point2<f32>,
        z: f32,
    ) -> Result<Point2<f32>, Error>;
    fn pixel_to_robot_with_x(
        &self,
        pixel_coordinates: Point2<f32>,
        x: f32,
    ) -> Result<Point3<f32>, Error>;
    fn robot_to_pixel(&self, robot_coordinates: Point3<f32>) -> Result<Point2<f32>, Error>;
    fn get_pixel_radius(
        &self,
        radius_in_robot_coordinates: f32,
        pixel_coordinates: Point2<f32>,
        resolution: Vector2<u32>,
    ) -> Result<f32, Error>;
}

impl Projection for CameraMatrix {
    fn pixel_to_camera(&self, pixel_coordinates: Point2<f32>) -> Vector3<f32> {
        vector![
            1.0,
            (self.optical_center.x - pixel_coordinates.x) / self.focal_length.x,
            (self.optical_center.y - pixel_coordinates.y) / self.focal_length.y
        ]
    }

    fn camera_to_pixel(&self, camera_ray: Vector3<f32>) -> Result<Point2<f32>, Error> {
        if camera_ray.x <= 0.0 {
            return Err(Error::BehindCamera);
        }
        Ok(point![
            self.optical_center.x - self.focal_length.x * camera_ray.y / camera_ray.x,
            self.optical_center.y - self.focal_length.y * camera_ray.z / camera_ray.x
        ])
    }

    fn pixel_to_ground(&self, pixel_coordinates: Point2<f32>) -> Result<Point2<f32>, Error> {
        self.pixel_to_ground_with_z(pixel_coordinates, 0.0)
    }

    fn pixel_to_ground_with_z(
        &self,
        pixel_coordinates: Point2<f32>,
        z: f32,
    ) -> Result<Point2<f32>, Error> {
        let camera_ray = self.pixel_to_camera(pixel_coordinates);
        let camera_ray_over_ground = self.camera_to_ground.rotation * camera_ray;
        if camera_ray_over_ground.z >= 0.0
            || camera_ray_over_ground.x.is_nan()
            || camera_ray_over_ground.y.is_nan()
            || camera_ray_over_ground.z.is_nan()
        {
            return Err(Error::AboveHorizon);
        }

        let distance_to_plane = z - self.camera_to_ground.translation.z;
        let slope = distance_to_plane / camera_ray_over_ground.z;
        let intersection_point =
            self.camera_to_ground.translation.vector + camera_ray_over_ground * slope;
        Ok(point![intersection_point.x, intersection_point.y])
    }

    fn ground_to_pixel(&self, ground_coordinates: Point2<f32>) -> Result<Point2<f32>, Error> {
        self.ground_with_z_to_pixel(ground_coordinates, 0.0)
    }

    fn ground_with_z_to_pixel(
        &self,
        ground_coordinates: Point2<f32>,
        z: f32,
    ) -> Result<Point2<f32>, Error> {
        self.camera_to_pixel(
            (self.ground_to_camera * point![ground_coordinates.x, ground_coordinates.y, z]).coords,
        )
    }

    fn pixel_to_robot_with_x(
        &self,
        pixel_coordinates: Point2<f32>,
        x: f32,
    ) -> Result<Point3<f32>, Error> {
        if x <= 0.0 {
            return Err(Error::BehindCamera);
        }

        let camera_ray = self.pixel_to_camera(pixel_coordinates);
        let camera_ray_over_robot = self.camera_to_robot.rotation * camera_ray;

        let distance_to_plane = x - self.camera_to_robot.translation.x;
        let slope = distance_to_plane / camera_ray_over_robot.x;

        let intersection_point =
            self.camera_to_robot.translation.vector + camera_ray_over_robot * slope;
        Ok(point![x, intersection_point.y, intersection_point.z])
    }

    fn robot_to_pixel(&self, robot_coordinates: Point3<f32>) -> Result<Point2<f32>, Error> {
        let camera_coordinates = self.robot_to_camera * robot_coordinates;
        self.camera_to_pixel(camera_coordinates.coords)
    }

    fn get_pixel_radius(
        &self,
        radius_in_robot_coordinates: f32,
        pixel_coordinates: Point2<f32>,
        resolution: Vector2<u32>,
    ) -> Result<f32, Error> {
        let robot_coordinates =
            self.pixel_to_ground_with_z(pixel_coordinates, radius_in_robot_coordinates)?;
        let camera_coordinates =
            self.ground_to_camera * point![robot_coordinates.x, robot_coordinates.y, 0.0];
        let distance = camera_coordinates.coords.norm();
        if distance <= radius_in_robot_coordinates {
            return Err(Error::TooClose);
        }
        let angle = (radius_in_robot_coordinates / distance).asin();
        Ok(resolution.y as f32 * angle / self.field_of_view.y)
    }
}
