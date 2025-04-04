import random
import time
import os

# Enum for Directions
class Dirs:
    Left = 0
    Right = 1
    Up = 2
    Down = 3
    Count = 4

class Edge:
    def __init__(self, c1, c2):
        self.cells = [c1, c2]
        self.active = True

    def other(self, cell):
        if cell.id == self.cells[0].id:
            return self.cells[1]
        return self.cells[0]

class Cell:
    def __init__(self, id):
        self.id = id
        self.visited = False
        self.edges = [None, None, None, None]  # Left, Right, Up, Down

class Maze:
    def __init__(self, rows, cols):
        self.rows = rows
        self.cols = cols
        self.cells = [Cell(i) for i in range(rows * cols)]
        self.start = None
        self.end = None
        
        for row in range(rows):
            for col in range(cols):
                idx = self.cell_index(row, col)
                cell = self.cells[idx]
                edges = cell.edges

                # Left
                left = self.cell_neighbour_index(row, col, Dirs.Left)
                if left is not None:
                    other = self.cells[left]
                    edge = Edge(cell, self.cells[left])
                    if other.edges[Dirs.Right] is not None:
                        edge = other.edges[Dirs.Right]
                    edges[Dirs.Left] = edge
                    other.edges[Dirs.Right] = edge

                # Right
                right = self.cell_neighbour_index(row, col, Dirs.Right)
                if right is not None:
                    other = self.cells[right]
                    edge = Edge(cell, self.cells[right])
                    if other.edges[Dirs.Left] is not None:
                        edge = other.edges[Dirs.Left]
                    edges[Dirs.Right] = edge
                    other.edges[Dirs.Left] = edge

                # Up
                up = self.cell_neighbour_index(row, col, Dirs.Up)
                if up is not None:
                    other = self.cells[up]
                    edge = Edge(cell, self.cells[up])
                    if other.edges[Dirs.Down] is not None:
                        edge = other.edges[Dirs.Down]
                    edges[Dirs.Up] = edge
                    other.edges[Dirs.Down] = edge

                # Down
                down = self.cell_neighbour_index(row, col, Dirs.Down)
                if down is not None:
                    other = self.cells[down]
                    edge = Edge(cell, self.cells[down])
                    if other.edges[Dirs.Up] is not None:
                        edge = other.edges[Dirs.Up]
                    edges[Dirs.Down] = edge
                    other.edges[Dirs.Up] = edge


    def gen(self, rng, start=None, end=None):
        _start = start or rng.randint(0, self.cols - 1)
        _end = end or rng.randint(0, self.cols - 1)
        
        first = self.cell_index(0, _start)
        self.start = first
        self.end = self.cell_index(self.rows - 1, _end)
        self.traverse(rng, self.cells[first], 0)

    def traverse(self, rng, cell, depth):
        cell.visited = True
        while True:
            other = self.find_random_unvisited_neighbour(rng, cell)
            if other:
                self.traverse(rng, other, depth + 1)
            else:
                return

    def find_random_unvisited_neighbour(self, rng, cell):
        other = rng.randint(0, Dirs.Count - 1)
        tries = 0
        while tries < Dirs.Count:
            edge = cell.edges[other]
            if not edge:
                other = (other + 1) % Dirs.Count
                tries += 1
                continue
            other_cell = edge.other(cell)
            if other_cell.visited:
                other = (other + 1) % Dirs.Count
                tries += 1
                continue
            edge.active = False
            return other_cell
        return None

    def cell_index(self, row, col):
        return row * self.cols + col

    def cell_neighbour_index(self, row, col, neighbour):
        if col > 0 and neighbour == Dirs.Left:
            return self.cell_index(row, col - 1)
        elif col < self.cols - 1 and neighbour == Dirs.Right:
            return self.cell_index(row, col + 1)
        elif row > 0 and neighbour == Dirs.Up:
            return self.cell_index(row - 1, col)
        elif row < self.rows - 1 and neighbour == Dirs.Down:
            return self.cell_index(row + 1, col)
        return None

    def print_visited(self):
        for row in range(self.rows):
            for col in range(self.cols):
                idx = self.cell_index(row, col)
                cell = self.cells[idx]
                print(f"{cell.visited} ", end="")
            print()

    def generate_maze_svg(self, filename):
        with open(filename, 'w') as file:
            file.write(f"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 {self.cols * 20} {self.rows * 20}'>")

            for row in range(self.rows):
                for col in range(self.cols):
                    idx = self.cell_index(row, col)
                    cell = self.cells[idx]
                    x = col * 20
                    y = row * 20
                    fill = "white" if not cell.visited else "black"
                    if self.start == cell.id:
                        fill = "red"
                    if self.end == cell.id:
                        fill = "green"
                    file.write(f"<rect x='{x}' y='{y}' width='20' height='20' fill='{fill}' stroke='gray' stroke-width='0.5' />")

            for row in range(self.rows):
                for col in range(self.cols):
                    idx = self.cell_index(row, col)
                    cell = self.cells[idx]
                    x = col * 20
                    y = row * 20
                    right = cell.edges[Dirs.Right]
                    if right and not right.active:
                        file.write(f"<line x1='{x + 20}' y1='{y + 1}' x2='{x + 20}' y2='{y + 19}' stroke='black' stroke-width='1' />")
                    down = cell.edges[Dirs.Down]
                    if down and not down.active:
                        file.write(f"<line x1='{x + 1}' y1='{y + 20}' x2='{x + 19}' y2='{y + 20}' stroke='black' stroke-width='1' />")
                    left = cell.edges[Dirs.Left]
                    if left and not left.active:
                        file.write(f"<line x1='{x}' y1='{y + 1}' x2='{x}' y2='{y + 19}' stroke='black' stroke-width='1' />")
                    up = cell.edges[Dirs.Up]
                    if up and not up.active:
                        file.write(f"<line x1='{x + 1}' y1='{y}' x2='{x + 19}' y2='{y}' stroke='black' stroke-width='1' />")

            file.write("</svg>")

def main():
    args = ["10", "10"]  # Replace this with actual argument parsing
    if len(args) < 2:
        print("Maze generator use - maze rows cols")
        return

    rows = int(args[0])
    cols = int(args[1])

    rng = random.Random()
    maze = Maze(rows, cols)

    start_time = time.perf_counter_ns()  # Record start time
    
    maze.gen(rng)  # Generate the maze
    
    end_time = time.perf_counter_ns()  # Record end time
    elapsed_time = end_time - start_time  # Calculate elapsed time
    
    print(f"Maze generation took {elapsed_time/1000:.4f} ms.")  # Print elapsed time
    
    maze.generate_maze_svg("maze.svg")  # Generate the maze SVG

if __name__ == "__main__":
    main()