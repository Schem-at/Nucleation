
/home/harrison/nucleation-perf/final-next-pgo/pgo:     file format elf64-x86-64


Disassembly of section .text:

0000000000099f20 <<mc_tick::behaviour::TickCtx>::drain>:
   99f20:	55                   	push   %rbp
   99f21:	41 57                	push   %r15
   99f23:	41 56                	push   %r14
   99f25:	41 55                	push   %r13
   99f27:	41 54                	push   %r12
   99f29:	53                   	push   %rbx
   99f2a:	48 81 ec d8 00 00 00 	sub    $0xd8,%rsp
   99f31:	4c 8b 37             	mov    (%rdi),%r14
   99f34:	48 8b 4f 08          	mov    0x8(%rdi),%rcx
   99f38:	48 8b 47 10          	mov    0x10(%rdi),%rax
   99f3c:	48 89 44 24 20       	mov    %rax,0x20(%rsp)
   99f41:	0f b6 47 18          	movzbl 0x18(%rdi),%eax
   99f45:	c6 47 18 fe          	movb   $0xfe,0x18(%rdi)
   99f49:	3c fe                	cmp    $0xfe,%al
   99f4b:	0f 84 65 06 00 00    	je     9a5b6 <<mc_tick::behaviour::TickCtx>::drain+0x696>
   99f51:	48 89 fb             	mov    %rdi,%rbx
   99f54:	48 89 4c 24 40       	mov    %rcx,0x40(%rsp)
   99f59:	88 44 24 05          	mov    %al,0x5(%rsp)
   99f5d:	48 8d 4f 19          	lea    0x19(%rdi),%rcx
   99f61:	8b 01                	mov    (%rcx),%eax
   99f63:	48 89 8c 24 d0 00 00 	mov    %rcx,0xd0(%rsp)
   99f6a:	00 
   99f6b:	8b 49 03             	mov    0x3(%rcx),%ecx
   99f6e:	89 4c 24 4b          	mov    %ecx,0x4b(%rsp)
   99f72:	89 44 24 48          	mov    %eax,0x48(%rsp)
   99f76:	48 8b 87 b0 00 00 00 	mov    0xb0(%rdi),%rax
   99f7d:	48 89 44 24 38       	mov    %rax,0x38(%rsp)
   99f82:	48 85 c0             	test   %rax,%rax
   99f85:	74 18                	je     99f9f <<mc_tick::behaviour::TickCtx>::drain+0x7f>
   99f87:	45 31 ff             	xor    %r15d,%r15d
   99f8a:	48 8b 8b a8 00 00 00 	mov    0xa8(%rbx),%rcx
   99f91:	48 8b 41 10          	mov    0x10(%rcx),%rax
   99f95:	48 85 c0             	test   %rax,%rax
   99f98:	74 2a                	je     99fc4 <<mc_tick::behaviour::TickCtx>::drain+0xa4>
   99f9a:	e9 1c 02 00 00       	jmp    9a1bb <<mc_tick::behaviour::TickCtx>::drain+0x29b>
   99f9f:	8a 44 24 05          	mov    0x5(%rsp),%al
   99fa3:	88 43 18             	mov    %al,0x18(%rbx)
   99fa6:	e9 0b 06 00 00       	jmp    9a5b6 <<mc_tick::behaviour::TickCtx>::drain+0x696>
   99fab:	0f 1f 44 00 00       	nopl   0x0(%rax,%rax,1)
   99fb0:	48 8b 8b a8 00 00 00 	mov    0xa8(%rbx),%rcx
   99fb7:	48 8b 41 10          	mov    0x10(%rcx),%rax
   99fbb:	48 85 c0             	test   %rax,%rax
   99fbe:	0f 85 f7 01 00 00    	jne    9a1bb <<mc_tick::behaviour::TickCtx>::drain+0x29b>
   99fc4:	49 8b 56 10          	mov    0x10(%r14),%rdx
   99fc8:	48 85 d2             	test   %rdx,%rdx
   99fcb:	0f 84 b3 05 00 00    	je     9a584 <<mc_tick::behaviour::TickCtx>::drain+0x664>
   99fd1:	49 8b 76 08          	mov    0x8(%r14),%rsi
   99fd5:	48 c1 e2 05          	shl    $0x5,%rdx
   99fd9:	48 8d 04 16          	lea    (%rsi,%rdx,1),%rax
   99fdd:	48 8b 4c 16 f8       	mov    -0x8(%rsi,%rdx,1),%rcx
   99fe2:	48 3b 4c 16 f0       	cmp    -0x10(%rsi,%rdx,1),%rcx
   99fe7:	0f 83 79 01 00 00    	jae    9a166 <<mc_tick::behaviour::TickCtx>::drain+0x246>
   99fed:	48 8b 50 e8          	mov    -0x18(%rax),%rdx
   99ff1:	48 89 ce             	mov    %rcx,%rsi
   99ff4:	48 c1 e6 04          	shl    $0x4,%rsi
   99ff8:	8b 7c 32 08          	mov    0x8(%rdx,%rsi,1),%edi
   99ffc:	89 7c 24 58          	mov    %edi,0x58(%rsp)
   9a000:	48 8b 3c 32          	mov    (%rdx,%rsi,1),%rdi
   9a004:	48 89 7c 24 50       	mov    %rdi,0x50(%rsp)
   9a009:	44 0f b6 64 32 0c    	movzbl 0xc(%rdx,%rsi,1),%r12d
   9a00f:	44 0f b6 6c 32 0d    	movzbl 0xd(%rdx,%rsi,1),%r13d
   9a015:	48 ff c1             	inc    %rcx
   9a018:	48 89 48 f8          	mov    %rcx,-0x8(%rax)
   9a01c:	8b 44 24 58          	mov    0x58(%rsp),%eax
   9a020:	89 44 24 18          	mov    %eax,0x18(%rsp)
   9a024:	48 8b 44 24 50       	mov    0x50(%rsp),%rax
   9a029:	48 89 44 24 10       	mov    %rax,0x10(%rsp)
   9a02e:	44 88 64 24 06       	mov    %r12b,0x6(%rsp)
   9a033:	41 80 e5 01          	and    $0x1,%r13b
   9a037:	44 88 6c 24 07       	mov    %r13b,0x7(%rsp)
   9a03c:	49 81 ff 40 42 0f 00 	cmp    $0xf4240,%r15
   9a043:	0f 84 c8 04 00 00    	je     9a511 <<mc_tick::behaviour::TickCtx>::drain+0x5f1>
   9a049:	48 8b 43 20          	mov    0x20(%rbx),%rax
   9a04d:	8b 4c 24 10          	mov    0x10(%rsp),%ecx
   9a051:	2b 48 18             	sub    0x18(%rax),%ecx
   9a054:	8b 74 24 14          	mov    0x14(%rsp),%esi
   9a058:	2b 70 1c             	sub    0x1c(%rax),%esi
   9a05b:	8b 50 40             	mov    0x40(%rax),%edx
   9a05e:	3b 70 44             	cmp    0x44(%rax),%esi
   9a061:	41 0f 92 c0          	setb   %r8b
   9a065:	39 d1                	cmp    %edx,%ecx
   9a067:	0f 83 11 02 00 00    	jae    9a27e <<mc_tick::behaviour::TickCtx>::drain+0x35e>
   9a06d:	8b 7c 24 18          	mov    0x18(%rsp),%edi
   9a071:	2b 78 20             	sub    0x20(%rax),%edi
   9a074:	3b 78 48             	cmp    0x48(%rax),%edi
   9a077:	41 0f 92 c1          	setb   %r9b
   9a07b:	45 84 c0             	test   %r8b,%r8b
   9a07e:	0f 84 02 02 00 00    	je     9a286 <<mc_tick::behaviour::TickCtx>::drain+0x366>
   9a084:	45 84 c9             	test   %r9b,%r9b
   9a087:	0f 84 01 02 00 00    	je     9a28e <<mc_tick::behaviour::TickCtx>::drain+0x36e>
   9a08d:	89 f6                	mov    %esi,%esi
   9a08f:	48 0f af 70 30       	imul   0x30(%rax),%rsi
   9a094:	41 89 f8             	mov    %edi,%r8d
   9a097:	4c 0f af c2          	imul   %rdx,%r8
   9a09b:	89 cf                	mov    %ecx,%edi
   9a09d:	4c 01 c7             	add    %r8,%rdi
   9a0a0:	48 01 f7             	add    %rsi,%rdi
   9a0a3:	48 8b 70 10          	mov    0x10(%rax),%rsi
   9a0a7:	48 39 f7             	cmp    %rsi,%rdi
   9a0aa:	0f 83 98 04 00 00    	jae    9a548 <<mc_tick::behaviour::TickCtx>::drain+0x628>
   9a0b0:	48 8b 40 08          	mov    0x8(%rax),%rax
   9a0b4:	0f b7 2c 78          	movzwl (%rax,%rdi,2),%ebp
   9a0b8:	48 8b 4c 24 38       	mov    0x38(%rsp),%rcx
   9a0bd:	48 83 7c 24 20 00    	cmpq   $0x0,0x20(%rsp)
   9a0c3:	0f 85 cc 01 00 00    	jne    9a295 <<mc_tick::behaviour::TickCtx>::drain+0x375>
   9a0c9:	48 8d 05 20 02 fe ff 	lea    -0x1fde0(%rip),%rax        # 7a2f0 <core::num::imp::flt2dec::strategy::dragon::POW5TO256+0x1908>
   9a0d0:	48 89 44 24 50       	mov    %rax,0x50(%rsp)
   9a0d5:	48 c7 44 24 58 14 00 	movq   $0x14,0x58(%rsp)
   9a0dc:	00 00 
   9a0de:	8b 05 04 df 10 00    	mov    0x10df04(%rip),%eax        # 1a7fe8 <mc_tick::diagnostics::var_os::MC_TICK_TRACE_NOTIFY+0x18>
   9a0e4:	85 c0                	test   %eax,%eax
   9a0e6:	0f 85 3f 01 00 00    	jne    9a22b <<mc_tick::behaviour::TickCtx>::drain+0x30b>
   9a0ec:	48 83 3d dc de 10 00 	cmpq   $0xffffffffffffffff,0x10dedc(%rip)        # 1a7fd0 <mc_tick::diagnostics::var_os::MC_TICK_TRACE_NOTIFY>
   9a0f3:	ff 
   9a0f4:	44 0f b7 e5          	movzwl %bp,%r12d
   9a0f8:	0f 85 6d 02 00 00    	jne    9a36b <<mc_tick::behaviour::TickCtx>::drain+0x44b>
   9a0fe:	4c 39 61 10          	cmp    %r12,0x10(%rcx)
   9a102:	0f 86 3e 01 00 00    	jbe    9a246 <<mc_tick::behaviour::TickCtx>::drain+0x326>
   9a108:	48 8b 41 08          	mov    0x8(%rcx),%rax
   9a10c:	41 c1 e4 04          	shl    $0x4,%r12d
   9a110:	4a 8b 3c 20          	mov    (%rax,%r12,1),%rdi
   9a114:	48 85 ff             	test   %rdi,%rdi
   9a117:	0f 84 29 01 00 00    	je     9a246 <<mc_tick::behaviour::TickCtx>::drain+0x326>
   9a11d:	4c 01 e0             	add    %r12,%rax
   9a120:	48 8b 40 08          	mov    0x8(%rax),%rax
   9a124:	80 7c 24 07 00       	cmpb   $0x0,0x7(%rsp)
   9a129:	75 35                	jne    9a160 <<mc_tick::behaviour::TickCtx>::drain+0x240>
   9a12b:	48 8b 40 20          	mov    0x20(%rax),%rax
   9a12f:	8b 4c 24 18          	mov    0x18(%rsp),%ecx
   9a133:	89 4c 24 58          	mov    %ecx,0x58(%rsp)
   9a137:	48 8b 4c 24 10       	mov    0x10(%rsp),%rcx
   9a13c:	48 89 4c 24 50       	mov    %rcx,0x50(%rsp)
   9a141:	0f b6 4c 24 06       	movzbl 0x6(%rsp),%ecx
   9a146:	48 89 de             	mov    %rbx,%rsi
   9a149:	48 8d 54 24 50       	lea    0x50(%rsp),%rdx
   9a14e:	ff d0                	call   *%rax
   9a150:	49 ff c7             	inc    %r15
   9a153:	e9 58 fe ff ff       	jmp    99fb0 <<mc_tick::behaviour::TickCtx>::drain+0x90>
   9a158:	0f 1f 84 00 00 00 00 	nopl   0x0(%rax,%rax,1)
   9a15f:	00 
   9a160:	48 8b 40 38          	mov    0x38(%rax),%rax
   9a164:	eb c9                	jmp    9a12f <<mc_tick::behaviour::TickCtx>::drain+0x20f>
   9a166:	48 ff c1             	inc    %rcx
   9a169:	48 89 48 f8          	mov    %rcx,-0x8(%rax)
   9a16d:	49 8b 46 10          	mov    0x10(%r14),%rax
   9a171:	48 85 c0             	test   %rax,%rax
   9a174:	0f 84 36 fe ff ff    	je     99fb0 <<mc_tick::behaviour::TickCtx>::drain+0x90>
   9a17a:	48 ff c8             	dec    %rax
   9a17d:	49 89 46 10          	mov    %rax,0x10(%r14)
   9a181:	49 8b 4e 08          	mov    0x8(%r14),%rcx
   9a185:	48 c1 e0 05          	shl    $0x5,%rax
   9a189:	48 8b 14 01          	mov    (%rcx,%rax,1),%rdx
   9a18d:	48 ff ca             	dec    %rdx
   9a190:	48 83 fa fd          	cmp    $0xfffffffffffffffd,%rdx
   9a194:	0f 87 16 fe ff ff    	ja     99fb0 <<mc_tick::behaviour::TickCtx>::drain+0x90>
   9a19a:	48 01 c1             	add    %rax,%rcx
   9a19d:	48 8b 79 08          	mov    0x8(%rcx),%rdi
   9a1a1:	ff 15 79 ca 10 00    	call   *0x10ca79(%rip)        # 1a6c20 <free@GLIBC_2.2.5>
   9a1a7:	48 8b 8b a8 00 00 00 	mov    0xa8(%rbx),%rcx
   9a1ae:	48 8b 41 10          	mov    0x10(%rcx),%rax
   9a1b2:	48 85 c0             	test   %rax,%rax
   9a1b5:	0f 84 09 fe ff ff    	je     99fc4 <<mc_tick::behaviour::TickCtx>::drain+0xa4>
   9a1bb:	48 ff c8             	dec    %rax
   9a1be:	48 89 41 10          	mov    %rax,0x10(%rcx)
   9a1c2:	48 8b 49 08          	mov    0x8(%rcx),%rcx
   9a1c6:	48 c1 e0 05          	shl    $0x5,%rax
   9a1ca:	4c 8b 24 01          	mov    (%rcx,%rax,1),%r12
   9a1ce:	48 8b 6c 01 08       	mov    0x8(%rcx,%rax,1),%rbp
   9a1d3:	0f 10 44 01 10       	movups 0x10(%rcx,%rax,1),%xmm0
   9a1d8:	0f 29 44 24 50       	movaps %xmm0,0x50(%rsp)
   9a1dd:	4d 8b 6e 10          	mov    0x10(%r14),%r13
   9a1e1:	4d 3b 2e             	cmp    (%r14),%r13
   9a1e4:	74 3b                	je     9a221 <<mc_tick::behaviour::TickCtx>::drain+0x301>
   9a1e6:	49 8b 46 08          	mov    0x8(%r14),%rax
   9a1ea:	4c 89 e9             	mov    %r13,%rcx
   9a1ed:	48 c1 e1 05          	shl    $0x5,%rcx
   9a1f1:	4c 89 24 08          	mov    %r12,(%rax,%rcx,1)
   9a1f5:	48 89 6c 08 08       	mov    %rbp,0x8(%rax,%rcx,1)
   9a1fa:	0f 28 44 24 50       	movaps 0x50(%rsp),%xmm0
   9a1ff:	0f 11 44 08 10       	movups %xmm0,0x10(%rax,%rcx,1)
   9a204:	49 ff c5             	inc    %r13
   9a207:	4d 89 6e 10          	mov    %r13,0x10(%r14)
   9a20b:	48 8b 8b a8 00 00 00 	mov    0xa8(%rbx),%rcx
   9a212:	48 8b 41 10          	mov    0x10(%rcx),%rax
   9a216:	48 85 c0             	test   %rax,%rax
   9a219:	0f 84 a5 fd ff ff    	je     99fc4 <<mc_tick::behaviour::TickCtx>::drain+0xa4>
   9a21f:	eb 9a                	jmp    9a1bb <<mc_tick::behaviour::TickCtx>::drain+0x29b>
   9a221:	4c 89 f7             	mov    %r14,%rdi
   9a224:	e8 57 a6 00 00       	call   a4880 <<alloc::raw_vec::RawVec<mc_tick::schedule::ScheduledTick>>::grow_one>
   9a229:	eb bb                	jmp    9a1e6 <<mc_tick::behaviour::TickCtx>::drain+0x2c6>
   9a22b:	48 8d 3d 9e dd 10 00 	lea    0x10dd9e(%rip),%rdi        # 1a7fd0 <mc_tick::diagnostics::var_os::MC_TICK_TRACE_NOTIFY>
   9a232:	48 8d 74 24 50       	lea    0x50(%rsp),%rsi
   9a237:	e8 40 16 07 00       	call   10b87c <<std::sync::once_lock::OnceLock<core::option::Option<std::ffi::os_str::OsString>>>::initialize::<<std::sync::once_lock::OnceLock<core::option::Option<std::ffi::os_str::OsString>>>::get_or_init<mc_tick::diagnostics::var_os::{closure#0}>::{closure#0}, !>>
   9a23c:	48 8b 4c 24 38       	mov    0x38(%rsp),%rcx
   9a241:	e9 a6 fe ff ff       	jmp    9a0ec <<mc_tick::behaviour::TickCtx>::drain+0x1cc>
   9a246:	66 85 ed             	test   %bp,%bp
   9a249:	0f 84 01 ff ff ff    	je     9a150 <<mc_tick::behaviour::TickCtx>::drain+0x230>
   9a24f:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
   9a254:	4c 8b 60 10          	mov    0x10(%rax),%r12
   9a258:	4c 3b 20             	cmp    (%rax),%r12
   9a25b:	0f 84 78 03 00 00    	je     9a5d9 <<mc_tick::behaviour::TickCtx>::drain+0x6b9>
   9a261:	48 8b 4c 24 40       	mov    0x40(%rsp),%rcx
   9a266:	48 8b 41 08          	mov    0x8(%rcx),%rax
   9a26a:	66 42 89 2c 60       	mov    %bp,(%rax,%r12,2)
   9a26f:	49 ff c4             	inc    %r12
   9a272:	4c 89 61 10          	mov    %r12,0x10(%rcx)
   9a276:	49 ff c7             	inc    %r15
   9a279:	e9 32 fd ff ff       	jmp    99fb0 <<mc_tick::behaviour::TickCtx>::drain+0x90>
   9a27e:	45 31 c0             	xor    %r8d,%r8d
   9a281:	e9 e7 fd ff ff       	jmp    9a06d <<mc_tick::behaviour::TickCtx>::drain+0x14d>
   9a286:	45 31 c9             	xor    %r9d,%r9d
   9a289:	e9 f6 fd ff ff       	jmp    9a084 <<mc_tick::behaviour::TickCtx>::drain+0x164>
   9a28e:	31 ed                	xor    %ebp,%ebp
   9a290:	e9 23 fe ff ff       	jmp    9a0b8 <<mc_tick::behaviour::TickCtx>::drain+0x198>
   9a295:	48 8b 83 b8 00 00 00 	mov    0xb8(%rbx),%rax
   9a29c:	48 89 44 24 30       	mov    %rax,0x30(%rsp)
   9a2a1:	48 8b 44 24 20       	mov    0x20(%rsp),%rax
   9a2a6:	48 8b 40 10          	mov    0x10(%rax),%rax
   9a2aa:	c7 44 24 28 00 00 00 	movl   $0x0,0x28(%rsp)
   9a2b1:	00 
   9a2b2:	48 89 44 24 08       	mov    %rax,0x8(%rsp)
   9a2b7:	48 85 c0             	test   %rax,%rax
   9a2ba:	74 2a                	je     9a2e6 <<mc_tick::behaviour::TickCtx>::drain+0x3c6>
   9a2bc:	48 8b 44 24 20       	mov    0x20(%rsp),%rax
   9a2c1:	48 8b 40 08          	mov    0x8(%rax),%rax
   9a2c5:	48 8b 4c 24 08       	mov    0x8(%rsp),%rcx
   9a2ca:	48 c1 e1 05          	shl    $0x5,%rcx
   9a2ce:	48 8b 54 24 30       	mov    0x30(%rsp),%rdx
   9a2d3:	48 39 54 08 e0       	cmp    %rdx,-0x20(%rax,%rcx,1)
   9a2d8:	75 0c                	jne    9a2e6 <<mc_tick::behaviour::TickCtx>::drain+0x3c6>
   9a2da:	48 01 c8             	add    %rcx,%rax
   9a2dd:	8b 40 e8             	mov    -0x18(%rax),%eax
   9a2e0:	ff c0                	inc    %eax
   9a2e2:	89 44 24 28          	mov    %eax,0x28(%rsp)
   9a2e6:	8b 44 24 18          	mov    0x18(%rsp),%eax
   9a2ea:	89 44 24 58          	mov    %eax,0x58(%rsp)
   9a2ee:	48 8b 44 24 10       	mov    0x10(%rsp),%rax
   9a2f3:	48 89 44 24 50       	mov    %rax,0x50(%rsp)
   9a2f8:	48 8b 44 24 20       	mov    0x20(%rsp),%rax
   9a2fd:	48 8b 4c 24 08       	mov    0x8(%rsp),%rcx
   9a302:	48 3b 08             	cmp    (%rax),%rcx
   9a305:	0f 84 dd 02 00 00    	je     9a5e8 <<mc_tick::behaviour::TickCtx>::drain+0x6c8>
   9a30b:	48 8b 74 24 20       	mov    0x20(%rsp),%rsi
   9a310:	48 8b 46 08          	mov    0x8(%rsi),%rax
   9a314:	48 8b 7c 24 08       	mov    0x8(%rsp),%rdi
   9a319:	48 89 f9             	mov    %rdi,%rcx
   9a31c:	48 c1 e1 05          	shl    $0x5,%rcx
   9a320:	48 8b 54 24 30       	mov    0x30(%rsp),%rdx
   9a325:	48 89 14 08          	mov    %rdx,(%rax,%rcx,1)
   9a329:	8b 54 24 28          	mov    0x28(%rsp),%edx
   9a32d:	89 54 08 08          	mov    %edx,0x8(%rax,%rcx,1)
   9a331:	48 8b 54 24 50       	mov    0x50(%rsp),%rdx
   9a336:	48 89 54 08 0c       	mov    %rdx,0xc(%rax,%rcx,1)
   9a33b:	8b 54 24 58          	mov    0x58(%rsp),%edx
   9a33f:	89 54 08 14          	mov    %edx,0x14(%rax,%rcx,1)
   9a343:	66 89 6c 08 18       	mov    %bp,0x18(%rax,%rcx,1)
   9a348:	8a 54 24 05          	mov    0x5(%rsp),%dl
   9a34c:	88 54 08 1a          	mov    %dl,0x1a(%rax,%rcx,1)
   9a350:	44 88 64 08 1b       	mov    %r12b,0x1b(%rax,%rcx,1)
   9a355:	44 88 6c 08 1c       	mov    %r13b,0x1c(%rax,%rcx,1)
   9a35a:	48 ff c7             	inc    %rdi
   9a35d:	48 89 7e 10          	mov    %rdi,0x10(%rsi)
   9a361:	48 8b 4c 24 38       	mov    0x38(%rsp),%rcx
   9a366:	e9 5e fd ff ff       	jmp    9a0c9 <<mc_tick::behaviour::TickCtx>::drain+0x1a9>
   9a36b:	48 8b 35 66 dc 10 00 	mov    0x10dc66(%rip),%rsi        # 1a7fd8 <mc_tick::diagnostics::var_os::MC_TICK_TRACE_NOTIFY+0x8>
   9a372:	48 8b 15 67 dc 10 00 	mov    0x10dc67(%rip),%rdx        # 1a7fe0 <mc_tick::diagnostics::var_os::MC_TICK_TRACE_NOTIFY+0x10>
   9a379:	4c 8d ac 24 b8 00 00 	lea    0xb8(%rsp),%r13
   9a380:	00 
   9a381:	4c 89 ef             	mov    %r13,%rdi
   9a384:	e8 47 f2 00 00       	call   a95d0 <<alloc::string::String>::from_utf8_lossy>
   9a389:	49 8b 45 08          	mov    0x8(%r13),%rax
   9a38d:	48 89 44 24 28       	mov    %rax,0x28(%rsp)
   9a392:	49 8b 75 10          	mov    0x10(%r13),%rsi
   9a396:	48 8d 7c 24 50       	lea    0x50(%rsp),%rdi
   9a39b:	ba 01 00 00 00       	mov    $0x1,%edx
   9a3a0:	b9 01 00 00 00       	mov    $0x1,%ecx
   9a3a5:	49 89 f5             	mov    %rsi,%r13
   9a3a8:	e8 43 09 00 00       	call   9acf0 <<alloc::raw_vec::RawVecInner>::try_allocate_in>
   9a3ad:	48 8d 4c 24 50       	lea    0x50(%rsp),%rcx
   9a3b2:	48 8b 41 08          	mov    0x8(%rcx),%rax
   9a3b6:	48 89 44 24 30       	mov    %rax,0x30(%rsp)
   9a3bb:	83 39 01             	cmpl   $0x1,(%rcx)
   9a3be:	0f 84 04 02 00 00    	je     9a5c8 <<mc_tick::behaviour::TickCtx>::drain+0x6a8>
   9a3c4:	48 8b 44 24 60       	mov    0x60(%rsp),%rax
   9a3c9:	48 89 44 24 08       	mov    %rax,0x8(%rsp)
   9a3ce:	4d 85 ed             	test   %r13,%r13
   9a3d1:	74 13                	je     9a3e6 <<mc_tick::behaviour::TickCtx>::drain+0x4c6>
   9a3d3:	48 8b 7c 24 08       	mov    0x8(%rsp),%rdi
   9a3d8:	48 8b 74 24 28       	mov    0x28(%rsp),%rsi
   9a3dd:	4c 89 ea             	mov    %r13,%rdx
   9a3e0:	ff 15 42 c8 10 00    	call   *0x10c842(%rip)        # 1a6c28 <memcpy@GLIBC_2.14>
   9a3e6:	48 83 bc 24 b8 00 00 	cmpq   $0x0,0xb8(%rsp)
   9a3ed:	00 00 
   9a3ef:	7e 0b                	jle    9a3fc <<mc_tick::behaviour::TickCtx>::drain+0x4dc>
   9a3f1:	48 8b 7c 24 28       	mov    0x28(%rsp),%rdi
   9a3f6:	ff 15 24 c8 10 00    	call   *0x10c824(%rip)        # 1a6c20 <free@GLIBC_2.2.5>
   9a3fc:	48 8d 44 24 10       	lea    0x10(%rsp),%rax
   9a401:	48 8d 54 24 50       	lea    0x50(%rsp),%rdx
   9a406:	48 89 02             	mov    %rax,(%rdx)
   9a409:	48 8d 05 90 7e 09 00 	lea    0x97e90(%rip),%rax        # 1322a0 <<i32 as core::fmt::Display>::fmt>
   9a410:	48 89 42 08          	mov    %rax,0x8(%rdx)
   9a414:	48 8d 4c 24 14       	lea    0x14(%rsp),%rcx
   9a419:	48 89 4a 10          	mov    %rcx,0x10(%rdx)
   9a41d:	48 89 42 18          	mov    %rax,0x18(%rdx)
   9a421:	48 8d 4c 24 18       	lea    0x18(%rsp),%rcx
   9a426:	48 89 4a 20          	mov    %rcx,0x20(%rdx)
   9a42a:	48 89 42 28          	mov    %rax,0x28(%rdx)
   9a42e:	48 8d 35 14 9d f7 ff 	lea    -0x862ec(%rip),%rsi        # 14149 <serde_json::ser::Formatter::write_char_escape::HEX_DIGITS+0x54a9>
   9a435:	48 8d bc 24 b8 00 00 	lea    0xb8(%rsp),%rdi
   9a43c:	00 
   9a43d:	e8 7e 75 05 00       	call   f19c0 <alloc::fmt::format::format_inner>
   9a442:	0f 10 84 24 b8 00 00 	movups 0xb8(%rsp),%xmm0
   9a449:	00 
   9a44a:	0f 29 84 24 a0 00 00 	movaps %xmm0,0xa0(%rsp)
   9a451:	00 
   9a452:	48 8b 84 24 c8 00 00 	mov    0xc8(%rsp),%rax
   9a459:	00 
   9a45a:	48 89 84 24 b0 00 00 	mov    %rax,0xb0(%rsp)
   9a461:	00 
   9a462:	31 c0                	xor    %eax,%eax
   9a464:	48 89 44 24 50       	mov    %rax,0x50(%rsp)
   9a469:	4c 89 6c 24 58       	mov    %r13,0x58(%rsp)
   9a46e:	48 8b 4c 24 08       	mov    0x8(%rsp),%rcx
   9a473:	48 89 4c 24 60       	mov    %rcx,0x60(%rsp)
   9a478:	4c 89 6c 24 68       	mov    %r13,0x68(%rsp)
   9a47d:	48 89 44 24 70       	mov    %rax,0x70(%rsp)
   9a482:	4c 89 6c 24 78       	mov    %r13,0x78(%rsp)
   9a487:	48 b8 3b 00 00 00 3b 	movabs $0x3b0000003b,%rax
   9a48e:	00 00 00 
   9a491:	48 89 84 24 80 00 00 	mov    %rax,0x80(%rsp)
   9a498:	00 
   9a499:	c6 84 24 88 00 00 00 	movb   $0x1,0x88(%rsp)
   9a4a0:	01 
   9a4a1:	66 c7 84 24 90 00 00 	movw   $0x1,0x90(%rsp)
   9a4a8:	00 01 00 
   9a4ab:	4c 8b 2d 86 c7 10 00 	mov    0x10c786(%rip),%r13        # 1a6c38 <bcmp@GLIBC_2.2.5>
   9a4b2:	48 8d 7c 24 50       	lea    0x50(%rsp),%rdi
   9a4b7:	e8 14 72 0b 00       	call   1516d0 <<core::str::iter::SplitInternal<char>>::next>
   9a4bc:	48 85 c0             	test   %rax,%rax
   9a4bf:	0f 84 ca 01 00 00    	je     9a68f <<mc_tick::behaviour::TickCtx>::drain+0x76f>
   9a4c5:	48 89 c7             	mov    %rax,%rdi
   9a4c8:	48 89 d6             	mov    %rdx,%rsi
   9a4cb:	e8 f0 54 0b 00       	call   14f9c0 <<str>::trim_matches::<<char>::is_whitespace>>
   9a4d0:	48 3b 94 24 b0 00 00 	cmp    0xb0(%rsp),%rdx
   9a4d7:	00 
   9a4d8:	75 d8                	jne    9a4b2 <<mc_tick::behaviour::TickCtx>::drain+0x592>
   9a4da:	48 8b b4 24 a8 00 00 	mov    0xa8(%rsp),%rsi
   9a4e1:	00 
   9a4e2:	48 89 c7             	mov    %rax,%rdi
   9a4e5:	41 ff d5             	call   *%r13
   9a4e8:	85 c0                	test   %eax,%eax
   9a4ea:	75 c6                	jne    9a4b2 <<mc_tick::behaviour::TickCtx>::drain+0x592>
   9a4ec:	48 8b 43 40          	mov    0x40(%rbx),%rax
   9a4f0:	4c 39 60 10          	cmp    %r12,0x10(%rax)
   9a4f4:	0f 86 fd 00 00 00    	jbe    9a5f7 <<mc_tick::behaviour::TickCtx>::drain+0x6d7>
   9a4fa:	48 8b 40 08          	mov    0x8(%rax),%rax
   9a4fe:	4b 8d 14 64          	lea    (%r12,%r12,2),%rdx
   9a502:	48 8b 4c d0 08       	mov    0x8(%rax,%rdx,8),%rcx
   9a507:	48 8b 44 d0 10       	mov    0x10(%rax,%rdx,8),%rax
   9a50c:	e9 f2 00 00 00       	jmp    9a603 <<mc_tick::behaviour::TickCtx>::drain+0x6e3>
   9a511:	48 8b 83 a8 00 00 00 	mov    0xa8(%rbx),%rax
   9a518:	4c 8b 78 08          	mov    0x8(%rax),%r15
   9a51c:	4c 8b 60 10          	mov    0x10(%rax),%r12
   9a520:	48 c7 40 10 00 00 00 	movq   $0x0,0x10(%rax)
   9a527:	00 
   9a528:	49 83 c7 08          	add    $0x8,%r15
   9a52c:	49 83 ec 01          	sub    $0x1,%r12
   9a530:	72 22                	jb     9a554 <<mc_tick::behaviour::TickCtx>::drain+0x634>
   9a532:	49 83 7f f8 00       	cmpq   $0x0,-0x8(%r15)
   9a537:	74 09                	je     9a542 <<mc_tick::behaviour::TickCtx>::drain+0x622>
   9a539:	49 8b 3f             	mov    (%r15),%rdi
   9a53c:	ff 15 de c6 10 00    	call   *0x10c6de(%rip)        # 1a6c20 <free@GLIBC_2.2.5>
   9a542:	49 83 c7 20          	add    $0x20,%r15
   9a546:	eb e4                	jmp    9a52c <<mc_tick::behaviour::TickCtx>::drain+0x60c>
   9a548:	48 8d 15 39 ad 10 00 	lea    0x10ad39(%rip),%rdx        # 1a5288 <__frame_dummy_init_array_entry+0x3708>
   9a54f:	e8 f7 1d 09 00       	call   12c34b <core::panicking::panic_bounds_check>
   9a554:	4d 8b 7e 08          	mov    0x8(%r14),%r15
   9a558:	4d 8b 66 10          	mov    0x10(%r14),%r12
   9a55c:	49 c7 46 10 00 00 00 	movq   $0x0,0x10(%r14)
   9a563:	00 
   9a564:	49 83 c7 08          	add    $0x8,%r15
   9a568:	49 83 ec 01          	sub    $0x1,%r12
   9a56c:	72 16                	jb     9a584 <<mc_tick::behaviour::TickCtx>::drain+0x664>
   9a56e:	49 83 7f f8 00       	cmpq   $0x0,-0x8(%r15)
   9a573:	74 09                	je     9a57e <<mc_tick::behaviour::TickCtx>::drain+0x65e>
   9a575:	49 8b 3f             	mov    (%r15),%rdi
   9a578:	ff 15 a2 c6 10 00    	call   *0x10c6a2(%rip)        # 1a6c20 <free@GLIBC_2.2.5>
   9a57e:	49 83 c7 20          	add    $0x20,%r15
   9a582:	eb e4                	jmp    9a568 <<mc_tick::behaviour::TickCtx>::drain+0x648>
   9a584:	4c 89 33             	mov    %r14,(%rbx)
   9a587:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
   9a58c:	48 89 43 08          	mov    %rax,0x8(%rbx)
   9a590:	48 8b 44 24 20       	mov    0x20(%rsp),%rax
   9a595:	48 89 43 10          	mov    %rax,0x10(%rbx)
   9a599:	0f b6 44 24 05       	movzbl 0x5(%rsp),%eax
   9a59e:	88 43 18             	mov    %al,0x18(%rbx)
   9a5a1:	8b 44 24 48          	mov    0x48(%rsp),%eax
   9a5a5:	8b 4c 24 4b          	mov    0x4b(%rsp),%ecx
   9a5a9:	48 8b 94 24 d0 00 00 	mov    0xd0(%rsp),%rdx
   9a5b0:	00 
   9a5b1:	89 4a 03             	mov    %ecx,0x3(%rdx)
   9a5b4:	89 02                	mov    %eax,(%rdx)
   9a5b6:	48 81 c4 d8 00 00 00 	add    $0xd8,%rsp
   9a5bd:	5b                   	pop    %rbx
   9a5be:	41 5c                	pop    %r12
   9a5c0:	41 5d                	pop    %r13
   9a5c2:	41 5e                	pop    %r14
   9a5c4:	41 5f                	pop    %r15
   9a5c6:	5d                   	pop    %rbp
   9a5c7:	c3                   	ret
   9a5c8:	48 8b 74 24 60       	mov    0x60(%rsp),%rsi
   9a5cd:	48 8b 7c 24 30       	mov    0x30(%rsp),%rdi
   9a5d2:	e8 a0 a8 08 00       	call   124e77 <alloc::raw_vec::handle_error>
   9a5d7:	0f 0b                	ud2
   9a5d9:	48 8b 7c 24 40       	mov    0x40(%rsp),%rdi
   9a5de:	e8 cd 43 02 00       	call   be9b0 <<alloc::raw_vec::RawVec<mc_tick::state::StateId>>::grow_one>
   9a5e3:	e9 79 fc ff ff       	jmp    9a261 <<mc_tick::behaviour::TickCtx>::drain+0x341>
   9a5e8:	48 8b 7c 24 20       	mov    0x20(%rsp),%rdi
   9a5ed:	e8 3e 2d 0b 00       	call   14d330 <<alloc::raw_vec::RawVec<mc_tick::timeline::PistonEvent>>::grow_one>
   9a5f2:	e9 14 fd ff ff       	jmp    9a30b <<mc_tick::behaviour::TickCtx>::drain+0x3eb>
   9a5f7:	b8 0d 00 00 00       	mov    $0xd,%eax
   9a5fc:	48 8d 0d 55 fc fd ff 	lea    -0x203ab(%rip),%rcx        # 7a258 <core::num::imp::flt2dec::strategy::dragon::POW5TO256+0x1870>
   9a603:	48 8d 93 b8 00 00 00 	lea    0xb8(%rbx),%rdx
   9a60a:	48 89 8c 24 b8 00 00 	mov    %rcx,0xb8(%rsp)
   9a611:	00 
   9a612:	48 89 84 24 c0 00 00 	mov    %rax,0xc0(%rsp)
   9a619:	00 
   9a61a:	48 8d 74 24 50       	lea    0x50(%rsp),%rsi
   9a61f:	48 89 16             	mov    %rdx,(%rsi)
   9a622:	48 8d 05 b7 b1 09 00 	lea    0x9b1b7(%rip),%rax        # 1357e0 <<u64 as core::fmt::Display>::fmt>
   9a629:	48 89 46 08          	mov    %rax,0x8(%rsi)
   9a62d:	48 8d 84 24 a0 00 00 	lea    0xa0(%rsp),%rax
   9a634:	00 
   9a635:	48 89 46 10          	mov    %rax,0x10(%rsi)
   9a639:	48 8d 05 d0 b7 09 00 	lea    0x9b7d0(%rip),%rax        # 135e10 <<alloc::string::String as core::fmt::Display>::fmt>
   9a640:	48 89 46 18          	mov    %rax,0x18(%rsi)
   9a644:	48 8d 44 24 07       	lea    0x7(%rsp),%rax
   9a649:	48 89 46 20          	mov    %rax,0x20(%rsi)
   9a64d:	48 8d 05 3c ca 08 00 	lea    0x8ca3c(%rip),%rax        # 127090 <<mc_tick::behaviour::UpdateKind as core::fmt::Debug>::fmt>
   9a654:	48 89 46 28          	mov    %rax,0x28(%rsi)
   9a658:	48 8d 44 24 06       	lea    0x6(%rsp),%rax
   9a65d:	48 89 46 30          	mov    %rax,0x30(%rsi)
   9a661:	48 8d 05 58 73 0b 00 	lea    0xb7358(%rip),%rax        # 1519c0 <<mc_tick::pos::Dir as core::fmt::Debug>::fmt>
   9a668:	48 89 46 38          	mov    %rax,0x38(%rsi)
   9a66c:	48 8d 84 24 b8 00 00 	lea    0xb8(%rsp),%rax
   9a673:	00 
   9a674:	48 89 46 40          	mov    %rax,0x40(%rsi)
   9a678:	48 8d 05 11 5c 02 00 	lea    0x25c11(%rip),%rax        # c0290 <<&str as core::fmt::Display>::fmt>
   9a67f:	48 89 46 48          	mov    %rax,0x48(%rsi)
   9a683:	48 8d 3d 3b 7d f7 ff 	lea    -0x882c5(%rip),%rdi        # 123c5 <serde_json::ser::Formatter::write_char_escape::HEX_DIGITS+0x3725>
   9a68a:	e8 01 64 06 00       	call   100a90 <std::io::stdio::_eprint>
   9a68f:	48 83 bc 24 a0 00 00 	cmpq   $0x0,0xa0(%rsp)
   9a696:	00 00 
   9a698:	74 0e                	je     9a6a8 <<mc_tick::behaviour::TickCtx>::drain+0x788>
   9a69a:	48 8b bc 24 a8 00 00 	mov    0xa8(%rsp),%rdi
   9a6a1:	00 
   9a6a2:	ff 15 78 c5 10 00    	call   *0x10c578(%rip)        # 1a6c20 <free@GLIBC_2.2.5>
   9a6a8:	48 83 7c 24 30 00    	cmpq   $0x0,0x30(%rsp)
   9a6ae:	48 8b 4c 24 38       	mov    0x38(%rsp),%rcx
   9a6b3:	0f 84 45 fa ff ff    	je     9a0fe <<mc_tick::behaviour::TickCtx>::drain+0x1de>
   9a6b9:	48 8b 7c 24 08       	mov    0x8(%rsp),%rdi
   9a6be:	ff 15 5c c5 10 00    	call   *0x10c55c(%rip)        # 1a6c20 <free@GLIBC_2.2.5>
   9a6c4:	48 8b 4c 24 38       	mov    0x38(%rsp),%rcx
   9a6c9:	e9 30 fa ff ff       	jmp    9a0fe <<mc_tick::behaviour::TickCtx>::drain+0x1de>
   9a6ce:	eb 1a                	jmp    9a6ea <<mc_tick::behaviour::TickCtx>::drain+0x7ca>
   9a6d0:	48 89 c3             	mov    %rax,%rbx
   9a6d3:	48 83 bc 24 b8 00 00 	cmpq   $0x0,0xb8(%rsp)
   9a6da:	00 00 
   9a6dc:	48 8b 44 24 28       	mov    0x28(%rsp),%rax
   9a6e1:	48 89 44 24 08       	mov    %rax,0x8(%rsp)
   9a6e6:	7f 26                	jg     9a70e <<mc_tick::behaviour::TickCtx>::drain+0x7ee>
   9a6e8:	eb 2f                	jmp    9a719 <<mc_tick::behaviour::TickCtx>::drain+0x7f9>
   9a6ea:	48 89 c3             	mov    %rax,%rbx
   9a6ed:	48 83 bc 24 a0 00 00 	cmpq   $0x0,0xa0(%rsp)
   9a6f4:	00 00 
   9a6f6:	74 0e                	je     9a706 <<mc_tick::behaviour::TickCtx>::drain+0x7e6>
   9a6f8:	48 8b bc 24 a8 00 00 	mov    0xa8(%rsp),%rdi
   9a6ff:	00 
   9a700:	ff 15 1a c5 10 00    	call   *0x10c51a(%rip)        # 1a6c20 <free@GLIBC_2.2.5>
   9a706:	48 83 7c 24 30 00    	cmpq   $0x0,0x30(%rsp)
   9a70c:	74 0b                	je     9a719 <<mc_tick::behaviour::TickCtx>::drain+0x7f9>
   9a70e:	48 8b 7c 24 08       	mov    0x8(%rsp),%rdi
   9a713:	ff 15 07 c5 10 00    	call   *0x10c507(%rip)        # 1a6c20 <free@GLIBC_2.2.5>
   9a719:	48 89 df             	mov    %rbx,%rdi
   9a71c:	e8 3f 64 10 00       	call   1a0b60 <_Unwind_Resume@plt>
   9a721:	48 89 c3             	mov    %rax,%rbx
   9a724:	eb e0                	jmp    9a706 <<mc_tick::behaviour::TickCtx>::drain+0x7e6>
   9a726:	48 89 6c 24 08       	mov    %rbp,0x8(%rsp)
   9a72b:	48 89 c3             	mov    %rax,%rbx
   9a72e:	4d 85 e4             	test   %r12,%r12
   9a731:	75 db                	jne    9a70e <<mc_tick::behaviour::TickCtx>::drain+0x7ee>
   9a733:	eb e4                	jmp    9a719 <<mc_tick::behaviour::TickCtx>::drain+0x7f9>
