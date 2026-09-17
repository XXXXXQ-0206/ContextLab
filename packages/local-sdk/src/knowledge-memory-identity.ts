import type {
  LocalKnowledgeCitationV1,
  LocalKnowledgeMemoryReplayProjectionV1
} from "./knowledge-memory-projection";

export const KNOWLEDGE_SCOPE_NAMESPACE_V5 = "1c2f6eb7-8c58-5e48-b815-30a1365c7418";
export const KNOWLEDGE_MEMORY_REPLAY_NAMESPACE_V5 = "325a50f1-9b23-55c8-8566-2a4b84a3cfd0";

type ReplayProjectionIdentity = Omit<LocalKnowledgeMemoryReplayProjectionV1, "id">;

export function knowledgeMemoryScopeForContext(contextId: string): string {
  return uuidV5(KNOWLEDGE_SCOPE_NAMESPACE_V5, `context:${contextId}`);
}

export function knowledgeMemoryReplayProjectionId(projection: ReplayProjectionIdentity): string {
  const identity: Uint8Array[] = [];
  appendText(identity, projection.schema_version);
  appendUuid(identity, projection.citation_projection_id);
  appendText(identity, projection.citation_projection_schema_version);
  appendUuid(identity, projection.knowledge_scope);
  appendUuid(identity, projection.retrieval_id);
  appendText(identity, projection.retrieval_version);
  appendU64(identity, projection.citations.length);
  for (const citation of projection.citations) appendCitation(identity, citation);
  appendUuid(identity, projection.memory_id);
  appendUuid(identity, projection.memory_scope);
  appendU64(identity, projection.memory_timeline_version);
  appendText(identity, projection.memory_capability_schema_version);
  appendText(identity, projection.retention_policy_version);
  appendText(identity, projection.retention_decision);
  appendText(identity, projection.replay_state);
  return uuidV5(
    KNOWLEDGE_MEMORY_REPLAY_NAMESPACE_V5,
    concat(identity)
  );
}

function appendCitation(identity: Uint8Array[], citation: LocalKnowledgeCitationV1): void {
  appendUuid(identity, citation.document_id);
  appendUuid(identity, citation.document_revision_id);
  appendUuid(identity, citation.chunk_id);
  appendText(identity, citation.source_version);
  appendText(identity, citation.chunking_version);
  appendU64(identity, citation.ordinal);
  appendU64(identity, citation.range.start_byte);
  appendU64(identity, citation.range.end_byte);
  appendText(identity, citation.content_fingerprint);
}

function appendText(identity: Uint8Array[], value: string): void {
  appendComponent(identity, new TextEncoder().encode(value));
}

function appendUuid(identity: Uint8Array[], value: string): void {
  appendComponent(identity, uuidToBytes(value));
}

function appendU64(identity: Uint8Array[], value: number): void {
  if (!Number.isSafeInteger(value) || value < 0) throw new TypeError("identity integer is invalid");
  const bytes = new Uint8Array(8);
  let remaining = BigInt(value);
  for (let index = bytes.length - 1; index >= 0; index -= 1) {
    bytes[index] = Number(remaining & 0xffn);
    remaining >>= 8n;
  }
  appendComponent(identity, bytes);
}

function appendComponent(identity: Uint8Array[], value: Uint8Array): void {
  const length = new Uint8Array(8);
  let remaining = BigInt(value.length);
  for (let index = length.length - 1; index >= 0; index -= 1) {
    length[index] = Number(remaining & 0xffn);
    remaining >>= 8n;
  }
  identity.push(length, value);
}

function concat(values: readonly Uint8Array[]): Uint8Array {
  const length = values.reduce((total, value) => total + value.length, 0);
  const result = new Uint8Array(length);
  let offset = 0;
  for (const value of values) {
    result.set(value, offset);
    offset += value.length;
  }
  return result;
}

export function uuidV5(namespace: string, name: string | Uint8Array): string {
  const input = concat([uuidToBytes(namespace), typeof name === "string" ? new TextEncoder().encode(name) : name]);
  const digest = sha1(input).slice(0, 16);
  digest[6] = (digest[6]! & 0x0f) | 0x50;
  digest[8] = (digest[8]! & 0x3f) | 0x80;
  return bytesToUuid(digest);
}

export const deriveKnowledgeScope = knowledgeMemoryScopeForContext;
export const deriveKnowledgeMemoryReplayProjectionId = knowledgeMemoryReplayProjectionId;

function uuidToBytes(value: string): Uint8Array {
  if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(value)) {
    throw new TypeError("identity UUID must be lowercase canonical UUID");
  }
  const hex = value.replaceAll("-", "");
  const bytes = new Uint8Array(16);
  for (let index = 0; index < bytes.length; index += 1) {
    bytes[index] = Number.parseInt(hex.slice(index * 2, index * 2 + 2), 16);
  }
  return bytes;
}

function bytesToUuid(value: Uint8Array): string {
  const hex = [...value].map((byte) => byte.toString(16).padStart(2, "0")).join("");
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

function sha1(input: Uint8Array): Uint8Array {
  const paddedLength = Math.ceil((input.length + 9) / 64) * 64;
  const message = new Uint8Array(paddedLength);
  message.set(input);
  message[input.length] = 0x80;
  const bitLength = BigInt(input.length) * 8n;
  for (let index = 0; index < 8; index += 1) {
    message[paddedLength - 1 - index] = Number((bitLength >> BigInt(index * 8)) & 0xffn);
  }

  let h0 = 0x67452301;
  let h1 = 0xefcdab89;
  let h2 = 0x98badcfe;
  let h3 = 0x10325476;
  let h4 = 0xc3d2e1f0;
  const schedule = new Uint32Array(80);
  for (let offset = 0; offset < message.length; offset += 64) {
    for (let index = 0; index < 16; index += 1) {
      const base = offset + index * 4;
      schedule[index] = (
        (message[base]! << 24)
        | (message[base + 1]! << 16)
        | (message[base + 2]! << 8)
        | message[base + 3]!
      ) >>> 0;
    }
    for (let index = 16; index < 80; index += 1) {
      schedule[index] = rotateLeft(schedule[index - 3]! ^ schedule[index - 8]! ^ schedule[index - 14]! ^ schedule[index - 16]!, 1);
    }
    let a = h0;
    let b = h1;
    let c = h2;
    let d = h3;
    let e = h4;
    for (let index = 0; index < 80; index += 1) {
      let functionValue: number;
      let constant: number;
      if (index < 20) {
        functionValue = (b & c) | ((~b) & d);
        constant = 0x5a827999;
      } else if (index < 40) {
        functionValue = b ^ c ^ d;
        constant = 0x6ed9eba1;
      } else if (index < 60) {
        functionValue = (b & c) | (b & d) | (c & d);
        constant = 0x8f1bbcdc;
      } else {
        functionValue = b ^ c ^ d;
        constant = 0xca62c1d6;
      }
      const temp = (rotateLeft(a, 5) + functionValue + e + constant + schedule[index]!) >>> 0;
      e = d;
      d = c;
      c = rotateLeft(b, 30);
      b = a;
      a = temp;
    }
    h0 = (h0 + a) >>> 0;
    h1 = (h1 + b) >>> 0;
    h2 = (h2 + c) >>> 0;
    h3 = (h3 + d) >>> 0;
    h4 = (h4 + e) >>> 0;
  }

  const digest = new Uint8Array(20);
  const words = [h0, h1, h2, h3, h4];
  words.forEach((word, index) => {
    digest[index * 4] = word >>> 24;
    digest[index * 4 + 1] = word >>> 16;
    digest[index * 4 + 2] = word >>> 8;
    digest[index * 4 + 3] = word;
  });
  return digest;
}

function rotateLeft(value: number, shift: number): number {
  return ((value << shift) | (value >>> (32 - shift))) >>> 0;
}
