# Prelude
The Web Content Accessibility Guidelines (WCAG) 2.0[^1] covers a wide range of recommendations for making Web content more accessible. Following these guidelines will make content accessible to a wider range of people with disabilities, including blindness and low vision, deafness and hearing loss, learning disabilities, cognitive limitations, limited movement, speech disabilities, photosensitivity and combinations of these. Following these guidelines will also often make Web content more usable to users in general.

This report was compiled to assess the degree of accessibility that the Shopwise[^2] service provides. It was assessed against Shopwise version 0.7.0.

Note: Due to it's WebAssembly implementation, some assistive technologies such as certain screen-readers that rely on reading the HTML of a webpage will not work on our system. This is unfortunate, but assistive technologies for WASM do exist.[^3]

# Conformance
There are five requirements that must be met in order for content to be classified as 'conforming' to WCAG 2.[^4]
> Sections formatted like this are quoted directly from the WCAG page.
## 1. Levels of Conformance
> One of the following levels of conformance is met in full.
>
> - For Level A conformance (the minimum level of conformance), the [web page](https://www.w3.org/WAI/WCAG21/Understanding/conformance#dfn-web-page) [satisfies](https://www.w3.org/WAI/WCAG21/Understanding/conformance#dfn-satisfies-a-success-criterion) all the Level A success criteria, or a [conforming alternate version](https://www.w3.org/WAI/WCAG21/Understanding/conformance#dfn-conforming-alternate-version) is provided.
> - For Level AA conformance, the web page satisfies all the Level A and Level AA success criteria, or a Level AA conforming alternate version is provided.
> - For Level AAA conformance, the web page satisfies all the Level A, Level AA and Level AAA success criteria, or a Level AAA conforming alternate version is provided. 
 
This report has assessed ShopWise as A-conformant, because it satisfies all Level A criteria.[^5] See [[#Success Criteria]] for a breakdown of each individual criterion, and the corresponding assessment against ShopWise.
## 2. Full Web Pages Only
> [Conformance](https://www.w3.org/WAI/WCAG21/Understanding/conformance#dfn-conformance) (and conformance level) is for full [web page(s)](https://www.w3.org/WAI/WCAG21/Understanding/conformance#dfn-web-page) only, and cannot be achieved if part of a web page is excluded.

This report assesses the entire ShopWise system in full.
## 3. Complete Processes
> When a [web page](https://www.w3.org/WAI/WCAG21/Understanding/conformance#dfn-web-page) is one of a series of web pages presenting a [process](https://www.w3.org/WAI/WCAG21/Understanding/conformance#dfn-process) (i.e., a sequence of steps that need to be completed in order to accomplish an activity), all web pages in the process conform at the specified level or better. (Conformance is not possible at a particular level if any page in the process does not conform at that level or better.)
>
> An online store has a series of pages that are used to select and purchase products. All pages in the series from start to finish (checkout) conform in order for any page that is part of the process to conform.

The ShopWise page is not a part of a larger process, so this requirement is fulfilled by default.
### 4. Only Accessibility-Supported Ways of Using Technologies
> Only [accessibility-supported](https://www.w3.org/WAI/WCAG21/Understanding/conformance#dfn-accessibility-supported) ways of using [technologies](https://www.w3.org/WAI/WCAG21/Understanding/conformance#dfn-technology) are [relied upon](https://www.w3.org/WAI/WCAG21/Understanding/conformance#dfn-relied-upon) to satisfy the success criteria. Any information or functionality that is provided in a way that is not accessibility supported is also available in a way that is accessibility supported. (See [Understanding accessibility support](https://www.w3.org/WAI/WCAG21/Understanding/conformance#accessibility-support).)

ShopWise does not provide any functionality or content in a manner that is unsupported by accessibility tools in such a manner that an alternate way of providing said functionality or content is required to use the system.

# 5. Non-Interference
> If [technologies](https://www.w3.org/WAI/WCAG21/Understanding/conformance#dfn-technology) are used in a way that is not [accessibility supported](https://www.w3.org/WAI/WCAG21/Understanding/conformance#dfn-accessibility-supported), or if they are used in a non-conforming way, then they do not block the ability of users to access the rest of the page. In addition, the [web page](https://www.w3.org/WAI/WCAG21/Understanding/conformance#dfn-web-page) as a whole continues to meet the conformance requirements under each of the following conditions:
>
>1. when any technology that is not [relied upon](https://www.w3.org/WAI/WCAG21/Understanding/conformance#dfn-relied-upon) is turned on in a user agent,
>2. when any technology that is not relied upon is turned off in a user agent, and
>3. when any technology that is not relied upon is not supported by a user agent
>
>In addition, the following success criteria apply to all content on the page, including content that is not otherwise relied upon to meet conformance, because failure to meet them could interfere with any use of the page:
>
>- **1.4.2 - Audio Control**,
>- **2.1.2 - No Keyboard Trap**,
>- **2.3.1 - Three Flashes or Below Threshold**, and
>- **2.2.2 - Pause, Stop, Hide**.

ShopWise does not rely on technologies that are used in a way that is not accessibility supported. In addition, ShopWise achieves success criteria [[#1.4.2 Audio Control]], [[#2.1.2 No Keyboard Trap]], [[#Guideline 2.3 - Seizures and Physical Reactions|2.3.1 Three Flashes or Below Threshold]], and [[#Guideline 2.2 - Enough Time|2.2.2 Pause, Stop, Hide]].
# Success Criteria
## Principle 1 - Perceivable
### Guideline 1.1 - Text alternatives
#### 1.1.1 Non-text content
The only non-text, non-decorative content on our website is the mapping interface, in which case a short text description can not serve the same purpose and present the same information.
### Guideline 1.2 - Time-based media
This entire section concerns audio and video time-based media, which our system does not have.
### Guideline 1.3 - Adaptable
#### 1.3.1 Info and Relationships
All information conveyed by structure in our system does so with standard elements like headings and are thus understandable by assistive technology. Relationships between data are clearly identified via hover text. 
#### 1.3.2 Meaningful Sequence
The only meaningful 'sequence' of our system is that the user inputs - which can be entered in in any order - are entered before the outputs (the map and route displays). However, the outputs do not and can not appear before all neccessary inputs are entered, so it is impossible for assistive technology to read it in the wrong order.
#### 1.3.3 Sensory Characteristics
The only instruction provided for understanding and operating the system that relies on sensory characteristics is the address bar, where a message tells the user to "enter your address above" if browser location access was denied. However, this guideline specifies that, in some languages (such as English) "above" refers not only to the physical location of some object, but also to the ordering. "Above", in the context of widgets on a webpage, can also refer to the "previous" object in a list. So, our system is compliant with this guideline.
### Guideline 1.4 - Distinguishable
#### 1.4.1 Use of Color
Our system does not use colour for anything other than decoration. The entire page is perfectly usable in grayscale.
#### 1.4.2 Audio Control
Covered in [[#Guideline 1.2 - Time-based media]]
#### 1.4.3 Contrast (Minimum)
The lowest contrast area on the page is the travel, cost, and distance breakdown on the route output screen:
![[Assets/Images/Pasted image 20261015145102.png|312]][^6]
Calculating the contrast ratio $\frac{(40.7 + 0.05)}{(12.3 + 0.05)} ≈ 3.29$ gives a ratio less than 4.5:1, which unfortunatley falls below the standard. However, this is the extreme example, and the rest of the page falls within the threshold.
#### 1.4.4 Resize Text
The webpage is still very usable at 200% magnification:
![[Assets/Images/Pasted image 20261015145738.png]]
#### 1.4.5 Images of Text
Images of text are not used in our system at all, except for in the logo. Text in logos is a noted exception by this guideline.
#### 1.4.6 Contrast (Enhanced)
Covered by [[#1.4.3 Contrast (Minimum)]].
#### 1.4.7 Low or No Background Audio
Covered by [[#Guideline 1.2 - Time-based media]].
#### 1.4.8 Visual Presentation
Our system does not contain paragraphs/blocks of text.
#### 1.4.9 Images of Text (No Exception)
Covered by [[#1.4.5 Images of Text]].
## Principle 2 - Operable
### Guideline 2.1 - Keyboard Accessible
#### 2.1.1 Keyboard
All functionality of the content is operable through a keyboard interface (tab-navigating the fields) without requiring specific timings for individual keystrokes.
#### 2.1.2 No Keyboard Trap
Keyboard focus can be moved to all components of the page using a keyboard interface, and focus can be moved away from all components using only a keyboard interface, and it does not require more than unmodified arrow or tab keys or other standard exit methods.
#### **2.1.3** Keyboard (No Exception)
Covered in [[#2.1.1 Keyboard]].
### Guideline 2.2 - Enough Time
This section is removed for brevity, since none of the website requires timing-based input whatsoever.
### Guideline 2.3 - Seizures and Physical Reactions
This section is removed for brevity, since none of the website has any flashing elements.

### Guideline **2.4** - Navigable
Provide ways to help users navigate, find content, and determine where they are.
#### 2.4.1 Bypass Blocks
Our system does not have content that is repeated on multiple web pages.
#### 2.4.2 Page Titled
Our system has a title:
![[Assets/Images/Pasted image 20261015155653.png|183]]
#### 2.4.3 Focus Order
Our system is designed (but not required) to be interacted with in a top-down, left-to-right order, and the tab-navigation reflects this. 
#### 2.4.4 Link Purpose (In Context)
The only links on our page are in the extra information section at the bottom right of the page, where the alt text describes exactly the webpage it links to:
![[Assets/Images/Pasted image 20261015152519.png|257]]
#### 2.4.5 Multiple Ways
Our system contains only one page.
#### 2.4.6 Headings and Labels
Headings and labels are used to describe topic and purpose.
#### 2.4.7 Focus Visible
The current focused element is outlined and highlighted, e.g.
![[Assets/Images/Pasted image 20261015152909.png|262]]
#### 2.4.8 Location
Covered by [[#2.4.5 Multiple Ways]]
#### 2.4.9 Link Purpose (Link Only)
Covered in [[#2.4.4 Link Purpose (In Context)]]
#### 2.4.10 Section Headings
Covered in [[#2.4.6 Headings and Labels]]
## Principle 3 - Understandable
### Guideline **3.1** - Readable
#### 3.1.1 Language of Page
Our system only supports English.
#### 3.1.2 Language of Parts
Covered in [[#3.1.1 Language of Page]].
#### 3.1.3 Unusual Words
At every point in the system, clarification of functionality and meaining is supplied via hover-text.
#### 3.1.4 Abbreviations
Covered in [[#3.1.3 Unusual Words]]
#### 3.1.5 Reading Level
Our system does not require reading ability more advanced than the lower secondary education level after removal of proper names and titles.
#### 3.1.6 Pronunciation
Our system does not rely on specific pronunciation of words where meaning of the words, in context, is ambiguous without knowing the pronunciation.
### Guideline 3.2 - Predictable
#### 3.2.1 On Focus
Nowhere in our system does a user interface component receiving focus initiate a change of context.
#### 3.2.2 On Input
Nowhere in our system does changing the setting of any user interface component automatically cause a change of context.
#### 3.2.3 Consistent Navigation
Covered by [[#2.4.5 Multiple Ways]]
#### 3.2.4 Consistent Identification
Covered by [[#2.4.5 Multiple Ways]]
#### 3.2.5 Change on Request
Our system does not possess a 'change of context' anywhere. The entire functionality of the system is visible on one layout of one screen.
### Guideline 3.3 - Input Assistance
#### **3.3.1** Error Identification
When input errors are detected, the item that is in error is identified and the error is described to the user in text, e.g
![[Assets/Images/Pasted image 20261015154519.png|243]]![[Assets/Images/Pasted image 20261015154449.png|250]]
Sometimes, the error message is not useful, for example, when no stores are selected:
![[Assets/Images/Pasted image 20261015160234.png|348]]
All known instances of this are new bugs or regressions, however, and are scheduled to be fixed.
#### 3.3.2 Labels or Instructions
Labels or instructions are provided when content requires user input, usually via hover text or labels, e.g.![[Assets/Images/Pasted image 20261015155324.png|230]]![[Assets/Images/Pasted image 20261015155139.png|170]]
![[Assets/Images/Pasted image 20261015155121.png|299]]
#### 3.3.3 Error Suggestion
There are no situations where an input error is automatically detected and suggestions for correction are known.
#### 3.3.4 Error Prevention (Legal, Financial, Data)
Our system does not: cause legal commitments, cause financial transactions, modify or delete user-controllable data in data storage systems, or  submit user test responses.
#### 3.3.5 Help
All elements provide additional information via hover text, e.g.
![[Assets/Images/Pasted image 20261015155345.png|213]]![[Assets/Images/Pasted image 20261015155334.png|261]]
![[Assets/Images/Pasted image 20261015155327.png|262]]
#### 3.3.6 Error Prevention
For all elements that require the user to submit information, the submission is both reversible, and checked for input errors.

E.g. entering text in a numbered field is rejected:
![[Assets/Images/Pasted image 20261015155523.png|196]] ![[Assets/Images/Pasted image 20261015155529.png|191]]

## Principle 4 - Robust
### Guideline 4.1 Compatible
#### 4.1.1 Parsing
Our system is not implemented in a markup language (such as HTML) but in WebAssembly.
#### 4.1.2 Name, Role, Value
For all user interface components, the name and role can be programmatically determined; states, properties, and values that can be set by the user can be programmatically set; and notification of changes to these items is available to user agents, including assistive technologies.

Due to our reliance on the egui[^7] framework, all user interface components are implemented using default, standard fields, such as text entries, sliders, buttons, etc. This allows us to fulfull this guideline by default.

[^1]: https://www.w3.org/TR/WCAG20/#intro-related-docs
[^2]: https://gitlab-web.ecs.vuw.ac.nz/course-work/engr301/2026/project1/team5/shopwise/
[^3]: https://github.com/ashu-choudhury/bit_sr
[^4]: https://www.w3.org/WAI/WCAG21/Understanding/conformance
[^5]: If ShopWise satisfied criteroion [[#1.4.3 Contrast (Minimum)]], (the only failing criterion) it would be AAA-conformant (the highest level). A [tracking issue exists](https://gitlab.ecs.vuw.ac.nz/course-work/engr301/2026/project1/team5/shopwise/-/work_items/116) to rectify this.
[^6]: [GIMP](https://www.gimp.org/) was used for this contrast comparison.
[^7]: https://github.com/emilk/egui