import "#io"
import "#window" as win
import "#math"
import "#memory"
import "#time"

struct Edge {
    a: int
    b: int

    new(a: int, b: int) {
        self.a = a
        self.b = b
    }
}

struct WireFrame {
    vertices: [Verticle]
    edges: [Edge]

    new(vertices: [Verticle], edges: [Edge]) {
        self.vertices = vertices
        self.edges = edges
    }
}

struct Verticle {
    x: float
    y: float
    z: float

    new(x: float, y: float, z: float) {
        self.x = x
        self.y = y
        self.z = z
    }
}

struct Camera {
    pos: Verticle
    rot: Verticle
    near: float
    far: float
    fov: float

    new(pos: Verticle, rot: Verticle) {
        self.pos = pos
        self.rot = rot
        self.near = 0.1
        self.far = 1000.0
        self.fov = 90.0
    }

    fun movex(self, x: float) {
        let pos = self.pos
        pos.x = pos.x + x
    }
    fun movey(self, y: float) {
        let pos = self.pos
        pos.y = pos.y + y
    }
    fun movez(self, z: float) {
        let pos = self.pos
        pos.z = pos.z + z
    }

    fun lookAt(self, x: float, y: float, z: float) {
        let pos = self.pos
        let rot = self.rot

        let dx = x - pos.x
        let dy = y - pos.y
        let dz = z - pos.z

        let distance = math.sqrt(dx * dx + dy * dy + dz * dz)

        let pitch = math.atan2(-dy, distance)
        let yaw = math.atan2(dx, dz)

        rot.x = pitch
        rot.y = yaw
    }

    fun print(self) {
        let pos = self.pos
        let rot = self.rot
        io.println("Camera: pos: (" + pos.x + ", " + pos.y + ", " + pos.z + ") rot: (" + rot.x + ", " + rot.y + ", " + rot.z + ")")
    }
}

fun cube(): WireFrame {
    let vertices = [
        Verticle(-1.0, -1.0, -1.0),
        Verticle(1.0, -1.0, -1.0),
        Verticle(1.0, 1.0, -1.0),
        Verticle(-1.0, 1.0, -1.0),
        Verticle(-1.0, -1.0, 1.0),
        Verticle(1.0, -1.0, 1.0),
        Verticle(1.0, 1.0, 1.0),
        Verticle(-1.0, 1.0, 1.0)
    ]

    let edges = [
        Edge(0, 1),
        Edge(1, 2),
        Edge(2, 3),
        Edge(3, 0),
        Edge(4, 5),
        Edge(5, 6),
        Edge(6, 7),
        Edge(7, 4),
        Edge(0, 4),
        Edge(1, 5),
        Edge(2, 6),
        Edge(3, 7)
    ]

    return WireFrame(vertices, edges)
}

fun project(camera: Camera, verticle: Verticle): Verticle {
    let x = verticle.x - camera.pos.x
    let y = verticle.y - camera.pos.y
    let z = verticle.z - camera.pos.z

    let xrot = x * math.cos(camera.rot.x) - z * math.sin(camera.rot.x)
    let yrot = y * math.cos(camera.rot.y) - z * math.sin(camera.rot.y)
    let zrot = z * math.cos(camera.rot.z) - z * math.sin(camera.rot.z)

    let xfov = xrot * camera.fov / zrot
    let yfov = yrot * camera.fov / zrot

    return Verticle(xfov, yfov, zrot)
}

fun drawWireFrame(ctx: win.Window, camera: Camera, wireframe: WireFrame) {
    let vertices = wireframe.vertices
    let edges = wireframe.edges

    let i = 0
    while i < edges.len() {
        let edge = edges[i]
        let a = vertices[edge.a]
        let b = vertices[edge.b]

        let aProj = project(camera, a)
        let bProj = project(camera, b)

        ctx.color(win.Color(255, 255, 255, 255))
        ctx.drawLine(aProj.x, aProj.y, bProj.x, bProj.y)

        i += 1
    } 
}

fun main() {
    let ctx = win.WinBuilder()
        .title("Wireframe")
        .width(800)
        .height(600)
        .default()
        .build()
    ctx.fps(60)

    let event: win.Event?

    ctx.background(win.Color(0, 0, 0, 255))


    let cube = cube()

    // have 3 cameras looking at the cube from different angles and distances
    // each second the view changes to the next camera
    let cameras = [
        Camera(Verticle(5, 3, 3), Verticle(0.0, 0.0, 0.0)),
        Camera(Verticle(10, 3, 3), Verticle(0.0, 0.0, 0.0)),
        Camera(Verticle(0, -10, 0), Verticle(0.0, 0.0, 0.0))
    ]

    // make the camera look at the cube
    let i = 0
    while i < cameras.len() {
        cameras[i].lookAt(0.0, 0.0, 0.0)
        i += 1
    }

    let clock = time.Clock()
    

    loop "main": {
        loop "event": {
            event = ctx.poll()
            if !event? {
                break "event"
            }
            if event.code() as int == win.Events.Closed {
                break "main"
            }
        }
        ctx.clear()

        let camera = cameras[clock.elapsed() as int % cameras.len()]
        drawWireFrame(ctx, camera, cube)
        
        

        memory.Gc.sweep()
        ctx.display()
    }
}