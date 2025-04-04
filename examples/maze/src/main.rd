import "#io"
import "#time"
import "#fs"
import "#string"

struct Maze {
    rows: int
    cols: int
    cells: [Cell]
    start: int?
    end: int?

    new(rows: int, cols: int) {
        self.rows = rows
        self.cols = cols
        let cells: [Cell] = []
        self.cells = cells
        let i = (rows * cols) - 1
        while i >= 0 {
            cells.push(Cell(i))
            i -= 1
        }
        let row = 0
        while row < rows {
            let col = 0
            while col < cols {
                i = self.cellIndex(row, col)
                let cell = cells[i]
                let edges = cell.edges
                let left = self.cellNeighbourIndex(row, col, Dirs.Left)
                if left? {
                    let other = self.cells[left]
                    let edge = Edge(cell, self.cells[left])
                    if other.edges[Dirs.Right]? {
                        edge = other.edges[Dirs.Right]
                    }
                    edges[Dirs.Left] = edge
                    other.edges[Dirs.Right] = edge
                }

                let right = self.cellNeighbourIndex(row, col, Dirs.Right)
                if right? {
                    let other = self.cells[right]
                    let edge = Edge(cell, self.cells[right])
                    if other.edges[Dirs.Left]? {
                        edge = other.edges[Dirs.Left]
                    }
                    edges[Dirs.Right] = edge
                    other.edges[Dirs.Left] = edge
                }
                let up = self.cellNeighbourIndex(row, col, Dirs.Up)
                if up? {
                    let other = self.cells[up]
                    let edge = Edge(cell, self.cells[up])
                    if other.edges[Dirs.Down]? {
                        edge = other.edges[Dirs.Down]
                    }
                    edges[Dirs.Up] = edge
                    other.edges[Dirs.Down] = edge
                }

                let down = self.cellNeighbourIndex(row, col, Dirs.Down)
                if down? {
                    let other = self.cells[down]
                    let edge = Edge(cell, self.cells[down])
                    if other.edges[Dirs.Up]? {
                        edge = other.edges[Dirs.Up]
                    }
                    edges[Dirs.Down] = edge
                    other.edges[Dirs.Up] = edge
                }

                col += 1
            }
            row += 1
        }
    }

    fun gen(self, rng: time.Rng, start: int?) {
        let _start = 0
        if start? {
            _start = start
        } else {
            _start = rng.range(0, self.cols)
        }
        let first = self.cellIndex(0, _start)
        self.start = first
        self.traverse(rng, self.cells[first], 0)
        let max = 0
        let i = 0
        while i < self.cols {
            let cell = self.cells[i]
            let maxCell = self.cells[max]
            if cell.depth >= maxCell.depth {
                max = i
            }
            i += 1
        }
        self.end = self.cellIndex(self.rows-1, max)
    }

    fun traverse(self, rng: time.Rng, cell: Cell, depth: int) {
        cell.visited = true
        cell.depth = depth
        // self.generateMazeSvg("maze.svg")
        // io.input()
        loop {
            let other = self.findRandomUnvisitedNeighbour(rng, cell)
            if other? {
                self.traverse(rng, other, depth + 1)
            }else {
                return
            }
        }
    }

    fun findRandomUnvisitedNeighbour(self, rng: time.Rng, cell: Cell): Cell? {
        let other = rng.range(0, Dirs.Count)
        let tries = 0
        loop {
            if tries == Dirs.Count {
                return
            }
            if rng.range(0, 50) == 0 {
                other = (other + 1) % Dirs.Count
                tries += 1
                continue
            }
            let edge = cell.edges[other]
            if !edge? {
                other = (other + 1) % Dirs.Count
                tries += 1
                continue
            }
            let otherCell = edge.other(cell)
            if otherCell.visited {
                other = (other + 1) % Dirs.Count
                tries += 1
                continue
            }
            edge.active = false
            return otherCell
        }
    }

    fun cellIndex(self, row: int, col: int): int {
        return row * self.cols + col
    }

    fun cellNeighbourIndex(self, row: int, col: int, neighbour: int): int? {
        if col > 0 && neighbour == Dirs.Left {
            return self.cellIndex(row, col - 1)
        } else if col < self.cols - 1 && neighbour == Dirs.Right {
            return self.cellIndex(row, col + 1)
        } else if row > 0 && neighbour == Dirs.Up {
            return self.cellIndex(row - 1, col)
        } else if row < self.rows - 1 && neighbour == Dirs.Down {
            return self.cellIndex(row + 1, col)
        }
        return null
    }


    fun printVisited(self) {
        let i = 0
        let row = 0
        while row < self.rows {
            let col = 0
            while col < self.cols {
                i = self.cellIndex(row, col)
                let cell = self.cells[i]
                io.print("" + cell.visited + " ")
                col += 1
            }
            io.println("")
            row += 1
        }
    }

    fun generateMazeSvg(self, filename: string) {
        // Clear the file before writing new content
        fs.fileWrite(filename, "")

        // Write the opening SVG tag to the file
        fs.fileAppend(filename, "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 " + (self.cols * 20) + " " + (self.rows * 20) + "'>")

        // Draw each cell
        let row = 0
        while row < self.rows {
            let col = 0
            while col < self.cols {
                let i = self.cellIndex(row, col)
                let cell = self.cells[i]
                let x = col * 20
                let y = row * 20
                let fill = "white"  // Default fill color for unvisited
                if cell.visited {
                    fill = "black"  // Change color to black for visited cells
                }
                if self.start == cell.id {
                    fill = "red"
                }
                if self.end == cell.id {
                    fill = "green"
                }
                fs.fileAppend(filename, "<rect x='" + x + "' y='" + y + "' width='20' height='20' fill='" + fill + "' stroke='gray' stroke-width='0.5' />")  // Added stroke to define the cell borders
                col += 1
            }
            row += 1
        }

        // Draw edges (connections between cells)
        row = 0
        while row < self.rows {
            let col = 0
            while col < self.cols {
                let i = self.cellIndex(row, col)
                let cell = self.cells[i]
                let x = col * 20
                let y = row * 20

                let right = cell.edges[Dirs.Right]
                if right? {
                    if !right.active {
                        let startX = x + 20
                        let startY = y + 1
                        let endX = x + 20
                        let endY = y + 19
                        fs.fileAppend(filename, "<line x1='" + startX + "' y1='" + startY + "' x2='" + endX + "' y2='" + endY + "' stroke='black' stroke-width='1' />")
                    }
                }
                let down = cell.edges[Dirs.Down]
                if down? {
                    if !down.active {
                        let startX = x + 1
                        let startY = y + 20
                        let endX = x + 19
                        let endY = y + 20
                        fs.fileAppend(filename, "<line x1='" + startX + "' y1='" + startY + "' x2='" + endX + "' y2='" + endY + "' stroke='black' stroke-width='1' />")
                    }
                }
                let left = cell.edges[Dirs.Left]
                if left? {
                    if !left.active {
                        let startX = x
                        let startY = y + 1
                        let endX = x
                        let endY = y + 19
                        fs.fileAppend(filename, "<line x1='" + startX + "' y1='" + startY + "' x2='" + endX + "' y2='" + endY + "' stroke='black' stroke-width='1' />")
                    }
                }
                let up = cell.edges[Dirs.Up]
                if up? {
                    if !up.active {
                        let startX = x + 1
                        let startY = y
                        let endX = x + 19
                        let endY = y
                        fs.fileAppend(filename, "<line x1='" + startX + "' y1='" + startY + "' x2='" + endX + "' y2='" + endY + "' stroke='black' stroke-width='1' />")
                    }
                }
                col += 1
            }
            row += 1
        }

        // Close SVG content
        fs.fileAppend(filename, "</svg>")
    }
}

struct Cell {
    id: int
    visited: bool
    depth: int
    edges: [Edge?]

    new(id: int) {
        self.depth = 0
        self.id = id
        self.visited = false
        self.edges = [null, null, null, null]
    }
}

struct Edge {
    cells: [Cell]
    active: bool

    new(c1: Cell, c2: Cell) {
        self.cells = [c1, c2]
        self.active = true
    }

    fun other(self, cell: Cell): Cell {
        let cells = self.cells
        let first = cells[0]
        let second = cells[1]
        if cell.id == first.id {
            return second
        }
        return first
    }
}

enum Dirs {
    Left
    Right
    Up
    Down
    Count
}

fun main() {
    let args = io.vmargs()
    if args.len() < 2 {
        io.println("Maze generator use - maze rows cols")
        return
    }
    let rows = string.parse(args[0])
    let cols = string.parse(args[1])
    let rng = time.Rng()
    let maze = Maze(rows, cols)
    let cells = maze.cells
    let start = time.Clock()
    maze.gen(rng, null)
    let elapsed = start.elapsed()
    io.println("Maze generation took: " + elapsed * 1000 + " ms")

    maze.generateMazeSvg("maze.svg")
}