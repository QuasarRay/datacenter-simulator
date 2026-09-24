Reminders:

1. Use formal verification via Verus and Kani, to prevent mistakes, accelerate development, and increase cost efficiency and reduce credit/token consumption. you are allowed and encouraged to add any formal verification tool alongside Verus and Kani if it helps increase cost efficiency and reduce credit/token consumption.


2. Accelerate development by making automation tools implement topologies by reading them from netbox.


3. use metaprogramming AGGRESSIVELY.

---

Project Goals:

1. Create an NVIDIA lab exam blueprint named NCP-Metablueprint, which integrates the blueprints for the exams NCP-AIO, NCP-AIN, and NCP-AII, and not only it accurately checks every item in all three blueprints meticulously, it also tests the relationship between the three blueprint and that every question requires knowledge of all three blueprints, and mastering one or two of the NCP reference blueprints, is not equal to mastering one-third or two-third of the NCP-Metablueprint, but rather it means mastering none of it. You are allowed and encouraged to go beyond the requirements of the original NCP-AIO, NCP-AIN, and NCP-AII exam blueprints in the field of GPU datacenter infrastructure when creating and documenting the NCP-Metablueprint, but do not miss any item from the original blueprints.

2. Then I would like you to use the existing automated exercise testing infrastructure in QuasarRay/rustlings.git (Your are allowed and encouraged to modify the rustlings source code) to create a comprehensive course + guided project + exercise bank (Black Box from the Student's perspective, they have to figure out the solution like real world scenarios) that fully covers the entire NVIDIA NCP-Metablueprint, while leveraging the existing datacenter-simulator capabilities to the fullest. the student will be working with real NVIDIA DeepOps throughout the courses, guided projects, and exercise labs. The current datacenter-simulator contains instructions to replace existing docker infra with incus system containers and instructs agents not to implement them. I would like you to bypass and remove those instructions and fully remove docker and containerlab from the project. the courses, guided projects and exercises will use a cluster that uses multiple incus system containers connected through patchbay+petgraph+netbox+rusternetes+KWOK+"NVIDIA datacenter compute/storage simulation + network emulation". since docker and kubernetes are part of NCP exams you may include them in the courses, but guided projects and exercises must fully replace docker with incus system containers (unless docker was a dependency of a higher level NVIDIA technology and avoiding docker were impossible), and fully replace Kubernetes with Rusternetes+KWOK(Unless unavoidable)

3. you are allowed, and encouraged to go beyond NVIDIA NCP-Metablueprint's requirements in the courses, guided projects and lab exercises, as long as the entire NCP-Metablueprint itself is covered. throughout the courses, guided projects and exercises, you will be constantly tempted to add skills and requirements from other vendors such as AMD, or even Cisco datacenter technologies, RHCAs, KVM Virtualization, CKA, CKNE, CNPE, WCA(wireshark certified analyst), DCIT, CCDE, vmware vcdx-vcf architect, terraform auth and ops(and blueprint equivalents in Pulumi) etc. because at some point a router/switch has to be used, linux/kubernetes adminstration and virtualization matter. feel free to add them when they solve a real world problem in GPU Datacenter Infrastructure administration.

4. Ideally the human language explanations in the courses should have english syntax, but low level programming language semantics, for example, Rust-CUDA semantics or but with idiomatic fluent and intuitive english

5. within the courses and guided projects, use diagrams and flowcharts heavily alongside hands on code and english explanations. the flowcharts and diagrams themselves must have low-level programming language semantics for example Rust-CUDA semantics but be intuitive, innovative, and Artistic, use effortlessly comprehensible and tangible(meaning uses physical/material objects and workflows symbolically) Symbolism, while remaining fully loyal to the actual behavior and internal architecture/design of NVIDIA infrastructure. for example, a very useful source of Artistic inspiration could be how Kubernetes architecture is symbolically explained through the workflows of the crew of a fleet in the sea in kodekloudhub/certified-kubernetes-administrator-course.git but do not limit yourself to the example. learn from it, and try to be better and more effortless to understand and more loyal to the reference behavior of NVIDIA infra.

6. it is preferred that in the courses, guided projects, and lab exercises, NVIDIA GPU datacenter infrastructure is managed via code, and even better if managed via metaprogramming features of either Rust, or Python, whichever fits the specific scenarios best. also heavily include utilities from libraries such as unpythonic/mcpyrate/lambars/karpal/etc(functional programming inspired from haskell, and macros inspired from lisp family and other languages as aggressive as Racket in metaprogramming) and utilities such as kr8s(and equivalents in Rust, and equivalents for fields other than kubernetes). prioritize these utilities whenever they make the solutions in courses, guided projects and lab exercises terser, and express more operations in fewer lines/characters. avoid CLI as much as possible, in the courses, guided projects, and lab exercises.

7. The courses must get more difficult incrementally

8. The guided projects must get more difficult incrementally

9. The exercises must get more difficult incrementally

10. formally prove the underlying Rustlings and the entire exercise labs bank correct using Verus and Kani. Use formal verifcation as a means of preventing credit consumption, and wasting effort fixing the project. do not treat formal verification as an extra burden to the project. make it a friend, an ally. make sure to fully leverage formal verification for the purpose of preventing future bugs from existing in the first place, rather than fixing them. it is preferred that the formal verification tools verify the code in the first round of development despite the specifications being maximally rigorous. turn formal verification tools into a teacher, a guide for yourself as the creator of the guided projects lab exercise bank. allow formal reasoning to make the process of creating the guided projects and the exercise labs more effortless for yourself. you are allowed to add other formal proofs alongside Verus and Kani if it makes the entire project easier, more reliable and reduces credit consumption even more.

11. try to use formal verification tools, Verus and Kani for preventing credits/tokens from being wasted. they should prevent you from using up all the credits/tokens for a mistake that is going to be removed. you are allowed to add other formal proofs alongside Verus and Kani if it makes the entire project easier, more reliable and reduces credit consumption even more.

12. make pull requests incrementally and stack them up, so that they can be pushed to main by pressing a single button. make sure to make a pull request after each small development cycle to ensure that no progress is lost in case your session is terminated or your internal container crashes. turn my github into your development environment.

Explanation of the meaning of the three terms, course, guided project and exercise:

. Course: a hands on guided project used to teach the students NCP-Metablueprint

. Guided project: a hands on guided project that assumes the student has mastered the entire NCP-Metablueprint via Courses and teaches How to use that knowledge to create real-world production ready projects, if any Guided project used a specific skill that were not already taught in courses, DO NOT REMOVE IT. add a course for it later that covers it. Guided projects teach how to combine the skills learned in courses in a variety of novel ways.

. Exercises: tasks that are done by the student independently of courses and guided projects but can be solved by finding a common pattern between the solution and what they previously experienced in guided projects and courses. if any exercise required a skill that was not taught in courses and guided projects, DO NOT REMOVE IT. add a course and a guided project later that covers it.

QuasarRay/datacenter-simulator.git

---

I emphasize:

Use formal verification via Verus and Kani, to prevent mistakes, accelerate development, and increase cost efficiency and reduce credit/token consumption. you are allowed and encouraged to add any formal verification tool alongside Verus and Kani if it helps increase cost efficiency and reduce credit/token consumption.
