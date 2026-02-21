/**
 * Three.js renderer for parsed .nucm chunks.
 */

import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';

/**
 * Create the Three.js scene, camera, renderer, and orbit controls.
 * @param {HTMLCanvasElement} canvas
 * @returns {{ scene, camera, renderer, controls }}
 */
export function createScene(canvas) {
  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
  renderer.setPixelRatio(window.devicePixelRatio);
  renderer.setSize(canvas.clientWidth, canvas.clientHeight);
  renderer.setClearColor(0x1a1a2e);

  const scene = new THREE.Scene();

  const camera = new THREE.PerspectiveCamera(
    60,
    canvas.clientWidth / canvas.clientHeight,
    0.1,
    50000
  );
  camera.position.set(50, 50, 50);

  const controls = new OrbitControls(camera, canvas);
  controls.enableDamping = true;
  controls.dampingFactor = 0.1;
  controls.screenSpacePanning = true;

  // Ambient light so MeshBasicMaterial colors are visible
  scene.add(new THREE.AmbientLight(0xffffff, 1.0));

  // Handle resize
  const onResize = () => {
    const w = canvas.clientWidth;
    const h = canvas.clientHeight;
    renderer.setSize(w, h);
    camera.aspect = w / h;
    camera.updateProjectionMatrix();
  };
  window.addEventListener('resize', onResize);

  // Animation loop
  const animate = () => {
    requestAnimationFrame(animate);
    controls.update();
    renderer.render(scene, camera);
  };
  animate();

  return { scene, camera, renderer, controls };
}

/**
 * Add a parsed chunk to the scene. Creates up to 3 meshes (opaque, cutout, transparent).
 * @param {THREE.Scene} scene
 * @param {object} chunk - Parsed chunk from nucm-parser
 * @returns {{ meshes: THREE.Mesh[], vertexCount: number, triangleCount: number }}
 */
export function addChunk(scene, chunk) {
  const meshes = [];
  let vertexCount = 0;
  let triangleCount = 0;

  // Create atlas texture for this chunk
  const atlasTexture = createAtlasTexture(chunk.atlas);

  const layerEntries = [
    { data: chunk.layers.opaque, name: 'opaque' },
    { data: chunk.layers.cutout, name: 'cutout' },
    { data: chunk.layers.transparent, name: 'transparent' },
  ];

  for (const { data, name } of layerEntries) {
    if (!data.positions || data.vertexCount === 0) continue;

    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute('position', new THREE.BufferAttribute(data.positions, 3));
    geometry.setAttribute('normal', new THREE.BufferAttribute(data.normals, 3));
    geometry.setAttribute('uv', new THREE.BufferAttribute(data.uvs, 2));

    // Three.js vertex colors expect RGB (3 components), but we have RGBA (4).
    // Extract RGB from the Float32Array.
    const rgb = new Float32Array(data.vertexCount * 3);
    for (let v = 0; v < data.vertexCount; v++) {
      rgb[v * 3] = data.colors[v * 4];
      rgb[v * 3 + 1] = data.colors[v * 4 + 1];
      rgb[v * 3 + 2] = data.colors[v * 4 + 2];
    }
    geometry.setAttribute('color', new THREE.BufferAttribute(rgb, 3));

    geometry.setIndex(new THREE.BufferAttribute(data.indices, 1));

    let material;
    if (name === 'opaque') {
      material = new THREE.MeshBasicMaterial({
        map: atlasTexture,
        vertexColors: true,
      });
    } else if (name === 'cutout') {
      material = new THREE.MeshBasicMaterial({
        map: atlasTexture,
        vertexColors: true,
        alphaTest: 0.5,
        side: THREE.DoubleSide,
      });
    } else {
      material = new THREE.MeshBasicMaterial({
        map: atlasTexture,
        vertexColors: true,
        transparent: true,
        depthWrite: false,
        side: THREE.DoubleSide,
      });
    }

    const mesh = new THREE.Mesh(geometry, material);
    scene.add(mesh);
    meshes.push(mesh);

    vertexCount += data.vertexCount;
    triangleCount += data.indexCount / 3;
  }

  return { meshes, vertexCount, triangleCount };
}

/**
 * Create a THREE.DataTexture from atlas pixel data.
 */
function createAtlasTexture(atlas) {
  if (!atlas.pixels || atlas.pixels.length === 0 || atlas.width === 0) {
    // Fallback white pixel
    const data = new Uint8Array([255, 255, 255, 255]);
    const tex = new THREE.DataTexture(data, 1, 1, THREE.RGBAFormat);
    tex.needsUpdate = true;
    return tex;
  }

  // Clone pixels since Three.js may take ownership
  const pixels = new Uint8Array(atlas.pixels);
  const tex = new THREE.DataTexture(pixels, atlas.width, atlas.height, THREE.RGBAFormat);
  tex.magFilter = THREE.NearestFilter;
  tex.minFilter = THREE.NearestFilter;
  tex.wrapS = THREE.RepeatWrapping;
  tex.wrapT = THREE.RepeatWrapping;
  tex.flipY = false;
  tex.needsUpdate = true;
  return tex;
}

/**
 * Remove all chunk meshes from the scene and dispose GPU resources.
 * @param {THREE.Scene} scene
 */
export function clearScene(scene) {
  const toRemove = [];
  scene.traverse((obj) => {
    if (obj.isMesh) toRemove.push(obj);
  });

  for (const mesh of toRemove) {
    scene.remove(mesh);
    mesh.geometry.dispose();
    if (mesh.material.map) mesh.material.map.dispose();
    mesh.material.dispose();
  }
}

/**
 * Fit the camera to encompass all meshes in the scene.
 * @param {THREE.PerspectiveCamera} camera
 * @param {OrbitControls} controls
 * @param {THREE.Scene} scene
 */
export function fitCamera(camera, controls, scene) {
  const box = new THREE.Box3();
  let hasMesh = false;

  scene.traverse((obj) => {
    if (obj.isMesh) {
      obj.geometry.computeBoundingBox();
      const meshBox = obj.geometry.boundingBox.clone();
      meshBox.applyMatrix4(obj.matrixWorld);
      box.union(meshBox);
      hasMesh = true;
    }
  });

  if (!hasMesh) return;

  const center = box.getCenter(new THREE.Vector3());
  const size = box.getSize(new THREE.Vector3());
  const maxDim = Math.max(size.x, size.y, size.z);
  const distance = maxDim * 1.5;

  controls.target.copy(center);
  camera.position.set(
    center.x + distance * 0.6,
    center.y + distance * 0.5,
    center.z + distance * 0.6
  );
  camera.near = maxDim * 0.001;
  camera.far = maxDim * 100;
  camera.updateProjectionMatrix();
  controls.update();
}
