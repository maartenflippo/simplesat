from argparse import ArgumentParser
from tqdm import tqdm
import csv
from datetime import timedelta
from pathlib import Path
import platform
import subprocess
import time
from typing import Iterable, Optional

TIMEOUT = timedelta(seconds=30)


def get_executable_path() -> Path:
    SOLVER_BINARY = Path("../target/release/simplesat")

    if platform.system() == "Windows":
        return SOLVER_BINARY.with_suffix(".exe")
    else:
        return SOLVER_BINARY


def solve(instance_file: Path) -> Optional[int]:
    """
    Run the solver on the CNF file, and report the real time in milliseconds.
    """
    assert instance_file.is_file(), \
        f"Instance file '{instance_file}' does not exist."

    solver_binary = get_executable_path()
    assert solver_binary.is_file(), \
        f"The solver binary '{solver_binary}' does not exist."

    try:
        start = time.perf_counter_ns()
        subprocess.run(
            [str(solver_binary), str(instance_file)], timeout=TIMEOUT.seconds, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

        # We don't care about nano seconds, just measure milliseconds.
        duration = (time.perf_counter_ns() - start) // 1_000_000

        return duration

    except subprocess.TimeoutExpired:
        return None


def main(folders: Iterable[Path], output_file: Path):
    """
    Run the solver for all the .cnf files in the given folders.

    For all the CNF files in the given folders, the simplesat solver will be 
    invoked, the real time required to solve the instance will be measured, and 
    the results are gathered in a CSV file. In the file, each row has two 
    columns: The first column is the instance name, and the second column the 
    real time (in milliseconds) needed to solve the instance, or '-' in case of
    a timeout.
    """
    if not output_file.parent.is_dir():
        output_file.parent.mkdir()

    with output_file.open('w', newline='') as f:
        writer = csv.writer(f)

        for folder in tqdm(folders):
            for instance_file in tqdm(folder.glob('*.cnf')):
                solve_time = solve(instance_file)

                writer.writerow([instance_file.as_posix(), str(solve_time)])


if __name__ == "__main__":
    parser = ArgumentParser(
        description="Benchmarking suite for the simplesat solver.")

    parser.add_argument("--output_file", type=Path,
                        default=Path(f"./results/benchmark-{int(time.time())}.csv"))
    parser.add_argument("folders", type=Path, nargs='+')

    args = parser.parse_args()

    main(args.folders, args.output_file)
