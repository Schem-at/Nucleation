/** The player: creative-flight camera plus the voxel raycast that decides
 * which block a click lands on.
 *
 * Controls are Minecraft's, because muscle memory is the whole point:
 * WASD to move, mouse to look, space/shift up and down, ctrl to sprint.
 * The raycast is the same grid march the engine uses for line of sight —
 * stepping cell to cell rather than testing meshes — so the block the
 * crosshair reports is the block the simulation will be told about, even
 * where the mesh is a torch or a lever that fills almost none of its cell.
 */

import * as THREE from "three";

export type Hit = { pos: [number, number, number]; face: [number, number, number] };

export class Player {
  camera: THREE.PerspectiveCamera;
  /** Look angles in radians; pitch clamped just shy of the poles. */
  yaw = 0.6;
  pitch = -0.25;
  speed = 12; // blocks per second, sprint doubles it
  private keys = new Set<string>();
  private locked = false;

  constructor(camera: THREE.PerspectiveCamera, dom: HTMLElement) {
    this.camera = camera;
    dom.addEventListener("click", () => {
      if (!this.locked) dom.requestPointerLock();
    });
    document.addEventListener("pointerlockchange", () => {
      this.locked = document.pointerLockElement === dom;
    });
    document.addEventListener("mousemove", (e) => {
      if (!this.locked) return;
      this.yaw -= e.movementX * 0.0022;
      this.pitch -= e.movementY * 0.0022;
      const limit = Math.PI / 2 - 0.001;
      this.pitch = Math.max(-limit, Math.min(limit, this.pitch));
    });
    window.addEventListener("keydown", (e) => {
      // Never swallow the browser's own chords.
      if (e.metaKey || e.altKey) return;
      this.keys.add(e.code);
      if (["Space", "ShiftLeft", "KeyW", "KeyA", "KeyS", "KeyD"].includes(e.code)) {
        e.preventDefault();
      }
    });
    window.addEventListener("keyup", (e) => this.keys.delete(e.code));
    window.addEventListener("blur", () => this.keys.clear());
  }

  get pointerLocked(): boolean {
    return this.locked;
  }

  /** Where the camera looks, as a unit vector. */
  direction(): THREE.Vector3 {
    return new THREE.Vector3(
      Math.cos(this.pitch) * Math.sin(this.yaw),
      Math.sin(this.pitch),
      Math.cos(this.pitch) * Math.cos(this.yaw),
    );
  }

  update(dt: number): void {
    const dir = this.direction();
    const forward = new THREE.Vector3(dir.x, 0, dir.z).normalize();
    // `forward × up` already points to the walker's right in a right-handed
    // Y-up frame: facing +Z, right is −X. Negating it swapped A and D.
    const right = new THREE.Vector3().crossVectors(forward, new THREE.Vector3(0, 1, 0));
    const move = new THREE.Vector3();
    if (this.keys.has("KeyW")) move.add(forward);
    if (this.keys.has("KeyS")) move.sub(forward);
    if (this.keys.has("KeyD")) move.add(right);
    if (this.keys.has("KeyA")) move.sub(right);
    if (this.keys.has("Space")) move.y += 1;
    if (this.keys.has("ShiftLeft") || this.keys.has("ShiftRight")) move.y -= 1;
    if (move.lengthSq() > 0) {
      const sprint = this.keys.has("ControlLeft") || this.keys.has("ControlRight") ? 3 : 1;
      move.normalize().multiplyScalar(this.speed * sprint * dt);
      this.camera.position.add(move);
    }
    this.camera.lookAt(this.camera.position.clone().add(dir));
  }

  /** March the view ray cell by cell (Amanatides–Woo) to the first solid
   * block, returning it and the face entered through. */
  pick(isSolid: (x: number, y: number, z: number) => boolean, reach = 64): Hit | null {
    // Blocks are **centred on integers**: block `i` occupies
    // [i-0.5, i+0.5]. Shifting the ray by half a block puts it in a space
    // where cell `i` spans [i, i+1], so the plain grid march below reads
    // out block coordinates directly. Without this every pick — and the
    // outline drawn from it — sits half a block off on all three axes.
    const origin = this.camera.position.clone().addScalar(0.5);
    const dir = this.direction();
    let cell: [number, number, number] = [
      Math.floor(origin.x),
      Math.floor(origin.y),
      Math.floor(origin.z),
    ];
    const step: [number, number, number] = [
      Math.sign(dir.x),
      Math.sign(dir.y),
      Math.sign(dir.z),
    ];
    const tDelta: [number, number, number] = [
      Math.abs(1 / dir.x),
      Math.abs(1 / dir.y),
      Math.abs(1 / dir.z),
    ];
    const dist = (o: number, c: number, s: number) => (s > 0 ? c + 1 - o : o - c);
    const tMax: [number, number, number] = [
      step[0] === 0 ? Infinity : dist(origin.x, cell[0], step[0]) * tDelta[0],
      step[1] === 0 ? Infinity : dist(origin.y, cell[1], step[1]) * tDelta[1],
      step[2] === 0 ? Infinity : dist(origin.z, cell[2], step[2]) * tDelta[2],
    ];
    let face: [number, number, number] = [0, 0, 0];
    let travelled = 0;
    for (let guard = 0; guard < reach * 3 && travelled <= reach; guard++) {
      if (isSolid(cell[0], cell[1], cell[2])) return { pos: [...cell], face };
      let axis = 0;
      if (tMax[1] < tMax[0]) axis = 1;
      if (tMax[2] < tMax[axis]) axis = 2;
      travelled = tMax[axis];
      cell[axis] += step[axis];
      tMax[axis] += tDelta[axis];
      face = [0, 0, 0];
      face[axis] = -step[axis];
    }
    return null;
  }

  /** Frame the whole build: back off along the view direction far enough to
   * see it all, looking at its middle. */
  frame(dims: [number, number, number]): void {
    const [dx, dy, dz] = dims;
    // Blocks sit on integers, so the middle of a `d`-wide build is (d-1)/2.
    const centre = new THREE.Vector3((dx - 1) / 2, (dy - 1) / 2, (dz - 1) / 2);
    const radius = Math.max(Math.hypot(dx, dy, dz) / 2, 4);
    this.yaw = 0.6;
    this.pitch = -0.35;
    this.camera.position.copy(centre).sub(this.direction().multiplyScalar(radius * 1.8));
    this.camera.lookAt(centre);
  }
}
